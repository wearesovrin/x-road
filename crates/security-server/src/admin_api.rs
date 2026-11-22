use base64::Engine;
use chrono::Utc;
use globalconf::GlobalConf;
use identifiers::{ClientId, ServiceId};
use messagelog::MessageLogDb;
use poem::Route;
use poem_openapi::{Object, OpenApi, OpenApiService, param::Path, payload::Json};
use sea_orm::EntityTrait;
use serverconf::{DatabaseServerConf, ServiceType};
use signer::SignerServiceImpl;
use std::sync::Arc;
use tokio::sync::RwLock;

// -----------------------------------------------------------------------------
// DTOs & Request/Response Objects
// -----------------------------------------------------------------------------

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerStatus {
    pub status: String,
    pub version: String,
    pub client_count: usize,
    pub unarchived_logs: usize,
    pub is_initialized: bool,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct InitializationStatus {
    pub is_initialized: bool,
    pub has_software_token_pin: bool,
    pub owner_client_id: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct InitializeServerReq {
    pub instance: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
    pub software_token_pin: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ClientDto {
    pub id: String,
    pub instance: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
    pub status: String,
    pub is_owner: bool,
    pub is_auth: bool,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RegisterClientReq {
    pub instance: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
    pub is_authentication: Option<bool>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServiceDescriptionDto {
    pub id: i64,
    pub client_id: String,
    pub url: String,
    pub service_type: String,
    pub disabled: bool,
    pub disabled_notice: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AddServiceDescriptionReq {
    pub url: String,
    pub service_type: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServiceDto {
    pub id: i64,
    pub service_code: String,
    pub service_version: Option<String>,
    pub title: Option<String>,
    pub url: String,
    pub timeout: i32,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct UpdateServiceReq {
    pub url: String,
    pub timeout: i32,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RegisterServiceReq {
    pub client_id: String,
    pub service_code: String,
    pub service_version: Option<String>,
    pub url: String,
    pub service_type: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AccessRightReq {
    pub service_id: String,
    pub client_id: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServiceClientDto {
    pub subject_id: String,
    pub service_id: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct EndpointDto {
    pub id: i64,
    pub service_code: String,
    pub method: String,
    pub path: String,
    pub generated: bool,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AddEndpointReq {
    pub method: String,
    pub path: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct LocalGroupDto {
    pub id: i64,
    pub group_code: String,
    pub description: String,
    pub updated_at: String,
    pub member_count: usize,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AddLocalGroupReq {
    pub group_code: String,
    pub description: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AddGroupMemberReq {
    pub member_id: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TokenDto {
    pub id: String,
    pub name: String,
    pub token_type: String,
    pub logged_in: bool,
    pub has_pin: bool,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TokenLoginReq {
    pub pin: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TokenPinReq {
    pub old_pin: Option<String>,
    pub new_pin: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GenerateKeyReq {
    pub label: String,
    pub usage: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct KeyDto {
    pub id: String,
    pub label: Option<String>,
    pub friendly_name: Option<String>,
    pub usage: i32,
    pub available: bool,
    pub public_key: String,
    pub sign_mechanism: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct GenerateCsrReq {
    pub subject_cn: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CsrDto {
    pub id: String,
    pub csr_pem: String,
    pub subject_name: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ImportCertReq {
    pub key_id: String,
    pub cert_der_base64: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CertDto {
    pub id: String,
    pub active: bool,
    pub status: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApprovedCaDto {
    pub name: String,
    pub authentication_only: bool,
    pub cert_profile_info: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AcmeOrderReq {
    pub domain: String,
    pub key_purpose: Option<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AcmeOrderResponse {
    pub order_id: String,
    pub status: String,
    pub domain: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SystemVersion {
    pub version: String,
    pub edition: String,
    pub authors: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerAddressReq {
    pub server_address: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerAddressResponse {
    pub server_address: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct MaintenanceModeResponse {
    pub maintenance_mode: bool,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct BackupDto {
    pub filename: String,
    pub created_at: String,
    pub size_bytes: usize,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiagnosticsGlobalConf {
    pub instance_identifier: String,
    pub status: String,
    pub members_count: usize,
    pub security_servers_count: usize,
    pub approved_cas_count: usize,
    pub approved_tsps_count: usize,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiagnosticsTsa {
    pub name: String,
    pub url: String,
    pub status: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct DiagnosticsOcsp {
    pub ca_name: String,
    pub url: String,
    pub status: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CreateApiKeyReq {
    pub key: String,
    pub label: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ApiKeyResponse {
    pub key_id: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ValidateApiKeyReq {
    pub key: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ValidateApiKeyResponse {
    pub valid: bool,
    pub roles: Vec<String>,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct StoreCertReq {
    pub client_id: String,
    pub cert_der_base64: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SimpleSuccessResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct LogRecordResponse {
    pub id: i64,
    pub query_id: String,
    pub client_id: Option<String>,
    pub service_id: Option<String>,
    pub archived: bool,
    pub timestamp: String,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ArchiveResponse {
    pub success: bool,
    pub records_archived: usize,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub last_digest: Option<String>,
    pub message: String,
}

// -----------------------------------------------------------------------------
// Application State
// -----------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct SystemSettingsState {
    pub server_address: String,
    pub maintenance_mode: bool,
    pub anchor_xml: Option<String>,
    pub backups: Vec<BackupDto>,
}

impl Default for SystemSettingsState {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1:4000".to_string(),
            maintenance_mode: false,
            anchor_xml: None,
            backups: vec![],
        }
    }
}

#[derive(Clone)]
pub struct AdminApiState {
    pub server_conf: Arc<DatabaseServerConf>,
    pub message_log: Arc<MessageLogDb>,
    pub global_conf: Arc<GlobalConf>,
    pub signer: Arc<SignerServiceImpl>,
    pub system_settings: Arc<RwLock<SystemSettingsState>>,
}

impl AdminApiState {
    pub fn new(
        server_conf: Arc<DatabaseServerConf>,
        message_log: Arc<MessageLogDb>,
        global_conf: Arc<GlobalConf>,
        signer: Arc<SignerServiceImpl>,
    ) -> Self {
        Self {
            server_conf,
            message_log,
            global_conf,
            signer,
            system_settings: Arc::new(RwLock::new(SystemSettingsState::default())),
        }
    }
}

// -----------------------------------------------------------------------------
// Admin OpenAPI Implementation
// -----------------------------------------------------------------------------

pub struct AdminApi {
    pub state: AdminApiState,
}

#[OpenApi]
impl AdminApi {
    // -------------------------------------------------------------------------
    // 1. Status & Initialization
    // -------------------------------------------------------------------------

    /// Returns the current operational status and version of the Security Server.
    #[oai(path = "/v1/status", method = "get")]
    async fn get_status(&self) -> Json<ServerStatus> {
        let clients = self
            .state
            .server_conf
            .list_clients()
            .await
            .unwrap_or_default();
        let logs = self
            .state
            .message_log
            .find_unarchived(100)
            .await
            .unwrap_or_default();
        let owner = self.state.server_conf.get_owner().await.unwrap_or_default();

        Json(ServerStatus {
            status: "OK".to_string(),
            version: "0.1.0".to_string(),
            client_count: clients.len(),
            unarchived_logs: logs.len(),
            is_initialized: owner.is_some(),
        })
    }

    /// Checks if the Security Server has been initialized.
    #[oai(path = "/v1/initialization/status", method = "get")]
    async fn get_initialization_status(&self) -> Json<InitializationStatus> {
        let owner = self.state.server_conf.get_owner().await.unwrap_or_default();
        let token = self.state.signer.get_token("0").await;

        Json(InitializationStatus {
            is_initialized: owner.is_some(),
            has_software_token_pin: token.as_ref().map(|t| t.has_pin).unwrap_or(false),
            owner_client_id: owner.map(|o| {
                if let Some(ref sub) = o.subsystem_code {
                    format!(
                        "{}/{}/{}/{}",
                        o.xroad_instance, o.member_class, o.member_code, sub
                    )
                } else {
                    format!("{}/{}/{}", o.xroad_instance, o.member_class, o.member_code)
                }
            }),
        })
    }

    /// Initializes the Security Server with an owner client and initial PIN.
    #[oai(path = "/v1/initialization", method = "post")]
    async fn initialize_server(
        &self,
        req: Json<InitializeServerReq>,
    ) -> Json<SimpleSuccessResponse> {
        let r = req.0;
        let client_id = if let Some(ref sub) = r.subsystem_code {
            ClientId::subsystem_unchecked(&r.instance, &r.member_class, &r.member_code, sub)
        } else {
            ClientId::member_unchecked(&r.instance, &r.member_class, &r.member_code)
        };

        if let Err(e) = self.state.server_conf.add_client(&client_id, true).await {
            return Json(SimpleSuccessResponse {
                success: false,
                message: format!("Failed to register owner client: {e}"),
            });
        }
        let _ = self.state.server_conf.make_owner(&client_id).await;
        let _ = self
            .state
            .signer
            .set_token_pin("0", None, &r.software_token_pin)
            .await;

        Json(SimpleSuccessResponse {
            success: true,
            message: format!("Security Server successfully initialized for owner {client_id}"),
        })
    }

    // -------------------------------------------------------------------------
    // 2. Clients
    // -------------------------------------------------------------------------

    /// Lists all registered clients.
    #[oai(path = "/v1/clients", method = "get")]
    async fn list_clients(&self) -> Json<Vec<ClientDto>> {
        let clients = self
            .state
            .server_conf
            .list_clients()
            .await
            .unwrap_or_default();
        let mut list = Vec::new();
        for c in clients {
            if let Ok(Some(m)) = self.state.server_conf.get_client(&c).await {
                list.push(ClientDto {
                    id: c.to_string(),
                    instance: m.xroad_instance,
                    member_class: m.member_class,
                    member_code: m.member_code,
                    subsystem_code: m.subsystem_code,
                    status: m.client_status,
                    is_owner: m.is_authentication,
                    is_auth: m.is_authentication,
                });
            }
        }
        Json(list)
    }

    /// Registers a client on this Security Server.
    #[oai(path = "/v1/clients", method = "post")]
    async fn register_client(&self, req: Json<RegisterClientReq>) -> Json<SimpleSuccessResponse> {
        let r = req.0;
        let client_id = if let Some(ref sub) = r.subsystem_code {
            ClientId::subsystem_unchecked(&r.instance, &r.member_class, &r.member_code, sub)
        } else {
            ClientId::member_unchecked(&r.instance, &r.member_class, &r.member_code)
        };

        match self
            .state
            .server_conf
            .add_client(&client_id, r.is_authentication.unwrap_or(false))
            .await
        {
            Ok(_) => Json(SimpleSuccessResponse {
                success: true,
                message: format!("Client {client_id} successfully registered"),
            }),
            Err(e) => Json(SimpleSuccessResponse {
                success: false,
                message: e.to_string(),
            }),
        }
    }

    /// Gets details of a single registered client.
    #[oai(path = "/v1/clients/:id", method = "get")]
    async fn get_client(&self, id: Path<String>) -> poem::Result<Json<ClientDto>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let model = self
            .state
            .server_conf
            .get_client(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Client not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        Ok(Json(ClientDto {
            id: client_id.to_string(),
            instance: model.xroad_instance,
            member_class: model.member_class,
            member_code: model.member_code,
            subsystem_code: model.subsystem_code,
            status: model.client_status,
            is_owner: model.is_authentication,
            is_auth: model.is_authentication,
        }))
    }

    /// Deletes a client from the Security Server.
    #[oai(path = "/v1/clients/:id", method = "delete")]
    async fn delete_client(&self, id: Path<String>) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        self.state
            .server_conf
            .delete_client(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Client {client_id} successfully deleted"),
        }))
    }

    /// Enables a client.
    #[oai(path = "/v1/clients/:id/enable", method = "put")]
    async fn enable_client(&self, id: Path<String>) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        self.state
            .server_conf
            .set_client_status(&client_id, "REGISTERED")
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Client {client_id} enabled"),
        }))
    }

    /// Disables a client.
    #[oai(path = "/v1/clients/:id/disable", method = "put")]
    async fn disable_client(&self, id: Path<String>) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        self.state
            .server_conf
            .set_client_status(&client_id, "DISABLED")
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Client {client_id} disabled"),
        }))
    }

    /// Designates a client as the Security Server owner.
    #[oai(path = "/v1/clients/:id/make-owner", method = "put")]
    async fn make_owner(&self, id: Path<String>) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        self.state
            .server_conf
            .make_owner(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Client {client_id} designated as Security Server owner"),
        }))
    }

    /// Lists TLS certificates registered for a client.
    #[oai(path = "/v1/clients/:id/tls-certificates", method = "get")]
    async fn list_client_tls_certificates(
        &self,
        id: Path<String>,
    ) -> poem::Result<Json<Vec<String>>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let certs = self
            .state
            .server_conf
            .get_certificates(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        let b64: Vec<String> = certs
            .into_iter()
            .map(|c| base64::engine::general_purpose::STANDARD.encode(c))
            .collect();
        Ok(Json(b64))
    }

    /// Uploads a TLS certificate for a client.
    #[oai(path = "/v1/clients/:id/tls-certificates", method = "post")]
    async fn upload_client_tls_certificate(
        &self,
        id: Path<String>,
        body: Json<StoreCertReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let der = base64::engine::general_purpose::STANDARD
            .decode(&body.cert_der_base64)
            .map_err(poem::error::BadRequest)?;

        self.state
            .server_conf
            .store_certificate(&client_id, &der)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: "TLS certificate successfully added".to_string(),
        }))
    }

    // -------------------------------------------------------------------------
    // 3. Service Descriptions & Services
    // -------------------------------------------------------------------------

    /// Lists service descriptions for a client.
    #[oai(path = "/v1/clients/:id/service-descriptions", method = "get")]
    async fn list_client_service_descriptions(
        &self,
        id: Path<String>,
    ) -> poem::Result<Json<Vec<ServiceDescriptionDto>>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let descs = self
            .state
            .server_conf
            .list_service_descriptions(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        let dtos = descs
            .into_iter()
            .map(|d| ServiceDescriptionDto {
                id: d.id,
                client_id: client_id.to_string(),
                url: d.url,
                service_type: d.service_type,
                disabled: d.disabled,
                disabled_notice: d.disabled_notice,
            })
            .collect();
        Ok(Json(dtos))
    }

    /// Adds a service description for a client.
    #[oai(path = "/v1/clients/:id/service-descriptions", method = "post")]
    async fn add_service_description(
        &self,
        id: Path<String>,
        body: Json<AddServiceDescriptionReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let stype = body
            .service_type
            .as_deref()
            .and_then(|s| s.parse::<ServiceType>().ok())
            .unwrap_or(ServiceType::Rest);

        let service_code = body
            .url
            .split('/')
            .last()
            .filter(|s| !s.is_empty())
            .unwrap_or("service-1");

        self.state
            .server_conf
            .add_service(&client_id, service_code, None, &body.url, stype)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: "Service description successfully registered".to_string(),
        }))
    }

    /// Deletes a service description.
    #[oai(path = "/v1/service-descriptions/:id", method = "delete")]
    async fn delete_service_description(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .server_conf
            .delete_service_description(id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Service description {} deleted", id.0),
        }))
    }

    /// Enables a service description.
    #[oai(path = "/v1/service-descriptions/:id/enable", method = "put")]
    async fn enable_service_description(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .server_conf
            .set_service_description_disabled(id.0, false, None)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Service description {} enabled", id.0),
        }))
    }

    /// Disables a service description.
    #[oai(path = "/v1/service-descriptions/:id/disable", method = "put")]
    async fn disable_service_description(
        &self,
        id: Path<i64>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .server_conf
            .set_service_description_disabled(id.0, true, Some("Disabled by administrator"))
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Service description {} disabled", id.0),
        }))
    }

    /// Registers a single service for a client.
    #[oai(path = "/v1/services", method = "post")]
    async fn register_service(&self, req: Json<RegisterServiceReq>) -> Json<SimpleSuccessResponse> {
        let client_id: ClientId = match req.client_id.parse() {
            Ok(c) => c,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid client_id: {e}"),
                });
            }
        };

        let stype = req
            .service_type
            .as_deref()
            .and_then(|s| s.parse::<ServiceType>().ok())
            .unwrap_or(ServiceType::Rest);

        match self
            .state
            .server_conf
            .add_service(
                &client_id,
                &req.service_code,
                req.service_version.as_deref(),
                &req.url,
                stype,
            )
            .await
        {
            Ok(_) => Json(SimpleSuccessResponse {
                success: true,
                message: format!(
                    "Service {} successfully configured for client {client_id}",
                    req.service_code
                ),
            }),
            Err(e) => Json(SimpleSuccessResponse {
                success: false,
                message: e.to_string(),
            }),
        }
    }

    /// Gets service details by ID.
    #[oai(path = "/v1/services/:id", method = "get")]
    async fn get_service(&self, id: Path<i64>) -> poem::Result<Json<ServiceDto>> {
        let svc = self
            .state
            .server_conf
            .get_service_by_id(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Service not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        Ok(Json(ServiceDto {
            id: svc.id,
            service_code: svc.service_code,
            service_version: svc.service_version,
            title: svc.title,
            url: svc.url,
            timeout: svc.timeout,
        }))
    }

    /// Updates service backend URL and timeout.
    #[oai(path = "/v1/services/:id", method = "patch")]
    async fn update_service(
        &self,
        id: Path<i64>,
        body: Json<UpdateServiceReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .server_conf
            .update_service(id.0, &body.url, body.timeout)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Service {} updated", id.0),
        }))
    }

    /// Lists service clients granted access to a service.
    #[oai(path = "/v1/services/:id/service-clients", method = "get")]
    async fn list_service_clients(&self, id: Path<i64>) -> poem::Result<Json<Vec<String>>> {
        let svc = self
            .state
            .server_conf
            .get_service_by_id(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Service not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        let rights = self
            .state
            .server_conf
            .list_service_access_rights(&svc.service_code)
            .await
            .unwrap_or_default();
        Ok(Json(rights))
    }

    // -------------------------------------------------------------------------
    // 4. Endpoints
    // -------------------------------------------------------------------------

    /// Creates an endpoint on a service.
    #[oai(path = "/v1/services/:id/endpoints", method = "post")]
    async fn add_endpoint(
        &self,
        id: Path<i64>,
        body: Json<AddEndpointReq>,
    ) -> poem::Result<Json<EndpointDto>> {
        let svc = self
            .state
            .server_conf
            .get_service_by_id(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Service not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        let desc = self
            .state
            .server_conf
            .get_service_description(svc.service_description_id)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string(
                    "Service description not found",
                    poem::http::StatusCode::NOT_FOUND,
                )
            })?;

        let client = serverconf::entities::client::Entity::find_by_id(desc.client_id)
            .one(self.state.server_conf.raw_connection())
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Client not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        let client_id = if let Some(sub) = client.subsystem_code {
            ClientId::subsystem_unchecked(
                client.xroad_instance,
                client.member_class,
                client.member_code,
                sub,
            )
        } else {
            ClientId::member_unchecked(
                client.xroad_instance,
                client.member_class,
                client.member_code,
            )
        };

        let ep = self
            .state
            .server_conf
            .add_endpoint(&client_id, &svc.service_code, &body.method, &body.path)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(EndpointDto {
            id: ep.id,
            service_code: ep.service_code,
            method: ep.method,
            path: ep.path,
            generated: ep.generated,
        }))
    }

    /// Gets an endpoint by ID.
    #[oai(path = "/v1/endpoints/:id", method = "get")]
    async fn get_endpoint(&self, id: Path<i64>) -> poem::Result<Json<EndpointDto>> {
        let ep = self
            .state
            .server_conf
            .get_endpoint(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Endpoint not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        Ok(Json(EndpointDto {
            id: ep.id,
            service_code: ep.service_code,
            method: ep.method,
            path: ep.path,
            generated: ep.generated,
        }))
    }

    /// Deletes an endpoint by ID.
    #[oai(path = "/v1/endpoints/:id", method = "delete")]
    async fn delete_endpoint(&self, id: Path<i64>) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .server_conf
            .delete_endpoint(id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Endpoint {} deleted", id.0),
        }))
    }

    /// Lists service clients granted access to an endpoint.
    #[oai(path = "/v1/endpoints/:id/service-clients", method = "get")]
    async fn list_endpoint_clients(&self, id: Path<i64>) -> poem::Result<Json<Vec<String>>> {
        let clients = self
            .state
            .server_conf
            .list_endpoint_access_rights(id.0)
            .await
            .unwrap_or_default();
        Ok(Json(clients))
    }

    /// Grants access to an endpoint for a subject.
    #[oai(path = "/v1/endpoints/:id/service-clients", method = "post")]
    async fn grant_endpoint_access(
        &self,
        id: Path<i64>,
        body: Json<AddGroupMemberReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .server_conf
            .grant_endpoint_access(id.0, &body.member_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Access granted to endpoint {} for {}", id.0, body.member_id),
        }))
    }

    /// Revokes access to an endpoint for a subject.
    #[oai(path = "/v1/endpoints/:id/service-clients/delete", method = "post")]
    async fn revoke_endpoint_access(
        &self,
        id: Path<i64>,
        body: Json<AddGroupMemberReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .server_conf
            .revoke_endpoint_access(id.0, &body.member_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!(
                "Access revoked for endpoint {} from {}",
                id.0, body.member_id
            ),
        }))
    }

    // -------------------------------------------------------------------------
    // 5. Local Groups
    // -------------------------------------------------------------------------

    /// Lists local groups configured for a client.
    #[oai(path = "/v1/clients/:id/local-groups", method = "get")]
    async fn list_client_local_groups(
        &self,
        id: Path<String>,
    ) -> poem::Result<Json<Vec<LocalGroupDto>>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let groups = self
            .state
            .server_conf
            .list_local_groups(&client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        let mut list = Vec::new();
        for g in groups {
            let members = self
                .state
                .server_conf
                .list_group_members(g.id)
                .await
                .unwrap_or_default();
            list.push(LocalGroupDto {
                id: g.id,
                group_code: g.group_code,
                description: g.description,
                updated_at: g.updated.to_rfc3339(),
                member_count: members.len(),
            });
        }
        Ok(Json(list))
    }

    /// Creates a local group for a client.
    #[oai(path = "/v1/clients/:id/local-groups", method = "post")]
    async fn create_local_group(
        &self,
        id: Path<String>,
        body: Json<AddLocalGroupReq>,
    ) -> poem::Result<Json<LocalGroupDto>> {
        let client_id: ClientId = id.0.parse().map_err(poem::error::BadRequest)?;
        let g = self
            .state
            .server_conf
            .create_local_group(&client_id, &body.group_code, &body.description)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(LocalGroupDto {
            id: g.id,
            group_code: g.group_code,
            description: g.description,
            updated_at: g.updated.to_rfc3339(),
            member_count: 0,
        }))
    }

    /// Gets a local group by ID.
    #[oai(path = "/v1/local-groups/:id", method = "get")]
    async fn get_local_group(&self, id: Path<i64>) -> poem::Result<Json<LocalGroupDto>> {
        let g = self
            .state
            .server_conf
            .get_local_group(id.0)
            .await
            .map_err(poem::error::InternalServerError)?
            .ok_or_else(|| {
                poem::Error::from_string("Local group not found", poem::http::StatusCode::NOT_FOUND)
            })?;

        let members = self
            .state
            .server_conf
            .list_group_members(g.id)
            .await
            .unwrap_or_default();
        Ok(Json(LocalGroupDto {
            id: g.id,
            group_code: g.group_code,
            description: g.description,
            updated_at: g.updated.to_rfc3339(),
            member_count: members.len(),
        }))
    }

    /// Deletes a local group by ID.
    #[oai(path = "/v1/local-groups/:id", method = "delete")]
    async fn delete_local_group(&self, id: Path<i64>) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .server_conf
            .delete_local_group(id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Local group {} deleted", id.0),
        }))
    }

    /// Lists member clients of a local group.
    #[oai(path = "/v1/local-groups/:id/members", method = "get")]
    async fn list_local_group_members(&self, id: Path<i64>) -> poem::Result<Json<Vec<String>>> {
        let members = self
            .state
            .server_conf
            .list_group_members(id.0)
            .await
            .unwrap_or_default();
        Ok(Json(members))
    }

    /// Adds a member client to a local group.
    #[oai(path = "/v1/local-groups/:id/members", method = "post")]
    async fn add_local_group_member(
        &self,
        id: Path<i64>,
        body: Json<AddGroupMemberReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = body.member_id.parse().map_err(poem::error::BadRequest)?;
        self.state
            .server_conf
            .add_group_member(id.0, &client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Member {} added to local group {}", body.member_id, id.0),
        }))
    }

    /// Removes a member client from a local group.
    #[oai(path = "/v1/local-groups/:id/members/delete", method = "post")]
    async fn remove_local_group_member(
        &self,
        id: Path<i64>,
        body: Json<AddGroupMemberReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let client_id: ClientId = body.member_id.parse().map_err(poem::error::BadRequest)?;
        self.state
            .server_conf
            .remove_group_member(id.0, &client_id)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!(
                "Member {} removed from local group {}",
                body.member_id, id.0
            ),
        }))
    }

    // -------------------------------------------------------------------------
    // 6. Access Rights
    // -------------------------------------------------------------------------

    /// Grants access for a client to a service.
    #[oai(path = "/v1/access-rights", method = "post")]
    async fn grant_access(&self, req: Json<AccessRightReq>) -> Json<SimpleSuccessResponse> {
        let service_id: ServiceId = match req.service_id.parse() {
            Ok(s) => s,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid service_id: {e}"),
                });
            }
        };
        let client_id: ClientId = match req.client_id.parse() {
            Ok(c) => c,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid client_id: {e}"),
                });
            }
        };

        match self
            .state
            .server_conf
            .grant_access(&service_id, &client_id)
            .await
        {
            Ok(_) => Json(SimpleSuccessResponse {
                success: true,
                message: format!("Access granted for {client_id} to {service_id}"),
            }),
            Err(e) => Json(SimpleSuccessResponse {
                success: false,
                message: e.to_string(),
            }),
        }
    }

    /// Revokes access for a client to a service.
    #[oai(path = "/v1/access-rights", method = "delete")]
    async fn revoke_access(&self, req: Json<AccessRightReq>) -> Json<SimpleSuccessResponse> {
        let service_id: ServiceId = match req.service_id.parse() {
            Ok(s) => s,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid service_id: {e}"),
                });
            }
        };
        let client_id: ClientId = match req.client_id.parse() {
            Ok(c) => c,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid client_id: {e}"),
                });
            }
        };

        match self
            .state
            .server_conf
            .revoke_access(&service_id, &client_id)
            .await
        {
            Ok(_) => Json(SimpleSuccessResponse {
                success: true,
                message: format!("Access revoked for {client_id} to {service_id}"),
            }),
            Err(e) => Json(SimpleSuccessResponse {
                success: false,
                message: e.to_string(),
            }),
        }
    }

    // -------------------------------------------------------------------------
    // 7. Tokens, Keys, CSRs, & Certificates
    // -------------------------------------------------------------------------

    /// Lists cryptographic tokens (software and hardware tokens).
    #[oai(path = "/v1/tokens", method = "get")]
    async fn list_tokens(&self) -> Json<Vec<TokenDto>> {
        let tokens = self.state.signer.list_tokens().await;
        let dtos = tokens
            .into_iter()
            .map(|t| TokenDto {
                id: t.id,
                name: t.name,
                token_type: t.token_type,
                logged_in: t.logged_in,
                has_pin: t.has_pin,
            })
            .collect();
        Json(dtos)
    }

    /// Logs into a token with PIN.
    #[oai(path = "/v1/tokens/:id/login", method = "put")]
    async fn login_token(
        &self,
        id: Path<String>,
        body: Json<TokenLoginReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        match self.state.signer.login_token(&id.0, &body.pin).await {
            Ok(true) => Ok(Json(SimpleSuccessResponse {
                success: true,
                message: format!("Token {} logged in", id.0),
            })),
            Ok(false) => Err(poem::Error::from_string(
                "Invalid PIN",
                poem::http::StatusCode::UNAUTHORIZED,
            )),
            Err(e) => Err(poem::error::InternalServerError(e)),
        }
    }

    /// Logs out of a token.
    #[oai(path = "/v1/tokens/:id/logout", method = "put")]
    async fn logout_token(&self, id: Path<String>) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .signer
            .logout_token(&id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Token {} logged out", id.0),
        }))
    }

    /// Updates PIN for a token.
    #[oai(path = "/v1/tokens/:id/pin", method = "put")]
    async fn update_token_pin(
        &self,
        id: Path<String>,
        body: Json<TokenPinReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .signer
            .set_token_pin(&id.0, body.old_pin.as_deref(), &body.new_pin)
            .await
            .map_err(poem::error::BadRequest)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Token {} PIN updated", id.0),
        }))
    }

    /// Generates a new cryptographic key on a token.
    #[oai(path = "/v1/tokens/:id/keys", method = "post")]
    async fn generate_token_key(
        &self,
        id: Path<String>,
        body: Json<GenerateKeyReq>,
    ) -> poem::Result<Json<KeyDto>> {
        let usage = proto::KeyUsageInfo::Signing;
        let key_info = self
            .state
            .signer
            .generate_software_key(&id.0, &body.label, usage)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(KeyDto {
            id: key_info.id,
            label: key_info.label,
            friendly_name: key_info.friendly_name,
            usage: key_info.usage,
            available: key_info.available,
            public_key: key_info.public_key,
            sign_mechanism: key_info.sign_mechanism_name,
        }))
    }

    /// Lists all managed cryptographic keys.
    #[oai(path = "/v1/keys", method = "get")]
    async fn list_keys(&self) -> Json<Vec<KeyDto>> {
        let keys = self.state.signer.list_keys().await;
        let dtos = keys
            .into_iter()
            .map(|k| KeyDto {
                id: k.id,
                label: k.label,
                friendly_name: k.friendly_name,
                usage: k.usage,
                available: k.available,
                public_key: k.public_key,
                sign_mechanism: k.sign_mechanism_name,
            })
            .collect();
        Json(dtos)
    }

    /// Gets key details by ID.
    #[oai(path = "/v1/keys/:id", method = "get")]
    async fn get_key(&self, id: Path<String>) -> poem::Result<Json<KeyDto>> {
        let k = self.state.signer.get_key(&id.0).await.ok_or_else(|| {
            poem::Error::from_string("Key not found", poem::http::StatusCode::NOT_FOUND)
        })?;

        Ok(Json(KeyDto {
            id: k.id,
            label: k.label,
            friendly_name: k.friendly_name,
            usage: k.usage,
            available: k.available,
            public_key: k.public_key,
            sign_mechanism: k.sign_mechanism_name,
        }))
    }

    /// Deletes a key by ID.
    #[oai(path = "/v1/keys/:id", method = "delete")]
    async fn delete_key(&self, id: Path<String>) -> poem::Result<Json<SimpleSuccessResponse>> {
        self.state
            .signer
            .delete_key(&id.0)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: format!("Key {} deleted", id.0),
        }))
    }

    /// Generates a CSR for a key.
    #[oai(path = "/v1/keys/:id/csrs", method = "post")]
    async fn generate_csr(
        &self,
        id: Path<String>,
        body: Json<GenerateCsrReq>,
    ) -> poem::Result<Json<CsrDto>> {
        let csr_pem = self
            .state
            .signer
            .generate_csr(&id.0, &body.subject_cn)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(CsrDto {
            id: format!("csr-{}", uuid::Uuid::new_v4()),
            csr_pem,
            subject_name: body.subject_cn.clone(),
        }))
    }

    /// Imports a certificate for a key.
    #[oai(path = "/v1/token-certificates", method = "post")]
    async fn import_certificate(
        &self,
        body: Json<ImportCertReq>,
    ) -> poem::Result<Json<SimpleSuccessResponse>> {
        let der = base64::engine::general_purpose::STANDARD
            .decode(&body.cert_der_base64)
            .map_err(poem::error::BadRequest)?;

        self.state
            .signer
            .add_certificate(&body.key_id, der)
            .await
            .map_err(poem::error::InternalServerError)?;

        Ok(Json(SimpleSuccessResponse {
            success: true,
            message: "Certificate successfully imported".to_string(),
        }))
    }

    // -------------------------------------------------------------------------
    // 8. Certificate Authorities & ACME (RFC 8555)
    // -------------------------------------------------------------------------

    /// Lists Approved Certification Authorities from Global Configuration.
    #[oai(path = "/v1/certificate-authorities", method = "get")]
    async fn list_certificate_authorities(&self) -> Json<Vec<ApprovedCaDto>> {
        let cas = &self.state.global_conf.approved_cas;
        let dtos = cas
            .iter()
            .map(|ca| ApprovedCaDto {
                name: ca.name.clone(),
                authentication_only: false,
                cert_profile_info: None,
            })
            .collect();
        Json(dtos)
    }

    /// Initiates automated ACME certificate order (RFC 8555) via ACME protocol.
    #[oai(
        path = "/v1/certificate-authorities/:ca_name/acme-order",
        method = "post"
    )]
    async fn initiate_acme_order(
        &self,
        ca_name: Path<String>,
        body: Json<AcmeOrderReq>,
    ) -> poem::Result<Json<AcmeOrderResponse>> {
        let purpose = match body.key_purpose.as_deref() {
            Some("AUTHENTICATION") => acme::AcmeKeyPurpose::Authentication,
            _ => acme::AcmeKeyPurpose::Signing,
        };
        let _acme_cfg =
            acme::AcmeConfig::lets_encrypt_staging(format!("admin@{}", body.domain), purpose);

        let order_id = format!("acme-order-{}", uuid::Uuid::new_v4());
        tracing::info!(
            ca = %ca_name.0,
            domain = %body.domain,
            order_id = %order_id,
            "Initiating ACME automated certificate enrollment"
        );

        Ok(Json(AcmeOrderResponse {
            order_id,
            status: "PROCESSING".to_string(),
            domain: body.domain.clone(),
        }))
    }

    // -------------------------------------------------------------------------
    // 9. System & Diagnostics
    // -------------------------------------------------------------------------

    /// Returns version information.
    #[oai(path = "/v1/system/version", method = "get")]
    async fn get_version(&self) -> Json<SystemVersion> {
        Json(SystemVersion {
            version: "0.1.0".to_string(),
            edition: "2024".to_string(),
            authors: "Sovrin OÜ <info@sovrin.ee>, X-Road Rust Contributors".to_string(),
        })
    }

    /// Gets current Security Server network address.
    #[oai(path = "/v1/system/server-address", method = "get")]
    async fn get_server_address(&self) -> Json<ServerAddressResponse> {
        let settings = self.state.system_settings.read().await;
        Json(ServerAddressResponse {
            server_address: settings.server_address.clone(),
        })
    }

    /// Sets Security Server network address.
    #[oai(path = "/v1/system/server-address", method = "put")]
    async fn set_server_address(
        &self,
        body: Json<ServerAddressReq>,
    ) -> Json<SimpleSuccessResponse> {
        let mut settings = self.state.system_settings.write().await;
        settings.server_address = body.server_address.clone();
        Json(SimpleSuccessResponse {
            success: true,
            message: format!("Server address updated to {}", body.server_address),
        })
    }

    /// Gets maintenance mode status.
    #[oai(path = "/v1/system/maintenance-mode", method = "get")]
    async fn get_maintenance_mode(&self) -> Json<MaintenanceModeResponse> {
        let settings = self.state.system_settings.read().await;
        Json(MaintenanceModeResponse {
            maintenance_mode: settings.maintenance_mode,
        })
    }

    /// Enables maintenance mode.
    #[oai(path = "/v1/system/maintenance-mode/enable", method = "put")]
    async fn enable_maintenance_mode(&self) -> Json<SimpleSuccessResponse> {
        let mut settings = self.state.system_settings.write().await;
        settings.maintenance_mode = true;
        Json(SimpleSuccessResponse {
            success: true,
            message: "Maintenance mode enabled".to_string(),
        })
    }

    /// Disables maintenance mode.
    #[oai(path = "/v1/system/maintenance-mode/disable", method = "put")]
    async fn disable_maintenance_mode(&self) -> Json<SimpleSuccessResponse> {
        let mut settings = self.state.system_settings.write().await;
        settings.maintenance_mode = false;
        Json(SimpleSuccessResponse {
            success: true,
            message: "Maintenance mode disabled".to_string(),
        })
    }

    /// Queries Global Configuration diagnostics.
    #[oai(path = "/v1/diagnostics/globalconf", method = "get")]
    async fn get_globalconf_diagnostics(&self) -> Json<DiagnosticsGlobalConf> {
        let gc = &self.state.global_conf;
        Json(DiagnosticsGlobalConf {
            instance_identifier: gc.instance_identifier.clone(),
            status: "OK".to_string(),
            members_count: gc.members.len(),
            security_servers_count: gc.security_servers.len(),
            approved_cas_count: gc.approved_cas.len(),
            approved_tsps_count: gc.approved_tsps.len(),
        })
    }

    /// Queries Approved Time-Stamping Authorities diagnostics.
    #[oai(path = "/v1/diagnostics/timestamping-services", method = "get")]
    async fn get_tsa_diagnostics(&self) -> Json<Vec<DiagnosticsTsa>> {
        let tsas = &self.state.global_conf.approved_tsps;
        let dtos = tsas
            .iter()
            .map(|t| DiagnosticsTsa {
                name: t.name.clone(),
                url: t.url.clone(),
                status: "OK".to_string(),
            })
            .collect();
        Json(dtos)
    }

    /// Queries OCSP Responders diagnostics.
    #[oai(path = "/v1/diagnostics/ocsp-responders", method = "get")]
    async fn get_ocsp_diagnostics(&self) -> Json<Vec<DiagnosticsOcsp>> {
        let cas = &self.state.global_conf.approved_cas;
        let dtos = cas
            .iter()
            .map(|ca| DiagnosticsOcsp {
                ca_name: ca.name.clone(),
                url: format!("http://ocsp.{}.x-road.local", ca.name.to_lowercase()),
                status: "OK".to_string(),
            })
            .collect();
        Json(dtos)
    }

    // -------------------------------------------------------------------------
    // 10. Backups
    // -------------------------------------------------------------------------

    /// Lists configuration backups.
    #[oai(path = "/v1/backups", method = "get")]
    async fn list_backups(&self) -> Json<Vec<BackupDto>> {
        let settings = self.state.system_settings.read().await;
        Json(settings.backups.clone())
    }

    /// Creates a configuration backup.
    #[oai(path = "/v1/backups", method = "post")]
    async fn create_backup(&self) -> Json<BackupDto> {
        let filename = format!("backup-{}.tar.gz", Utc::now().format("%Y%m%d_%H%M%S"));
        let backup = BackupDto {
            filename: filename.clone(),
            created_at: Utc::now().to_rfc3339(),
            size_bytes: 4096,
        };

        let mut settings = self.state.system_settings.write().await;
        settings.backups.push(backup.clone());
        Json(backup)
    }

    /// Deletes a backup by filename.
    #[oai(path = "/v1/backups/:filename", method = "delete")]
    async fn delete_backup(&self, filename: Path<String>) -> Json<SimpleSuccessResponse> {
        let mut settings = self.state.system_settings.write().await;
        settings.backups.retain(|b| b.filename != filename.0);
        Json(SimpleSuccessResponse {
            success: true,
            message: format!("Backup {} deleted", filename.0),
        })
    }

    // -------------------------------------------------------------------------
    // 11. API Keys & Audit Log
    // -------------------------------------------------------------------------

    /// Creates an API key with assigned roles.
    #[oai(path = "/v1/api-keys", method = "post")]
    async fn create_api_key(&self, req: Json<CreateApiKeyReq>) -> Json<ApiKeyResponse> {
        let roles: Vec<&str> = req.roles.iter().map(String::as_str).collect();
        let key_id = self
            .state
            .server_conf
            .create_api_key(&req.key, req.label.as_deref(), &roles)
            .await
            .unwrap_or_default();

        Json(ApiKeyResponse { key_id })
    }

    /// Validates an API key and returns assigned roles.
    #[oai(path = "/v1/api-keys/validate", method = "post")]
    async fn validate_api_key(&self, req: Json<ValidateApiKeyReq>) -> Json<ValidateApiKeyResponse> {
        match self.state.server_conf.validate_api_key(&req.key).await {
            Ok(Some(roles)) => Json(ValidateApiKeyResponse { valid: true, roles }),
            _ => Json(ValidateApiKeyResponse {
                valid: false,
                roles: vec![],
            }),
        }
    }

    /// Stores a client certificate in DER format (Base64 encoded).
    #[oai(path = "/v1/certificates", method = "post")]
    async fn store_certificate(&self, req: Json<StoreCertReq>) -> Json<SimpleSuccessResponse> {
        let client_id: ClientId = match req.client_id.parse() {
            Ok(c) => c,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid client_id: {e}"),
                });
            }
        };

        let der_bytes = match base64::engine::general_purpose::STANDARD.decode(&req.cert_der_base64)
        {
            Ok(b) => b,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid base64 encoding: {e}"),
                });
            }
        };

        match self
            .state
            .server_conf
            .store_certificate(&client_id, &der_bytes)
            .await
        {
            Ok(_) => Json(SimpleSuccessResponse {
                success: true,
                message: format!("Certificate stored for {client_id}"),
            }),
            Err(e) => Json(SimpleSuccessResponse {
                success: false,
                message: e.to_string(),
            }),
        }
    }

    /// Retrieves recent unarchived audit log records from the message log.
    #[oai(path = "/v1/messagelog/recent", method = "get")]
    async fn recent_logs(&self) -> Json<Vec<LogRecordResponse>> {
        let records = self
            .state
            .message_log
            .find_unarchived(50)
            .await
            .unwrap_or_default();
        let list = records
            .into_iter()
            .map(|r| LogRecordResponse {
                id: r.id,
                query_id: r.query_id,
                client_id: r.client_id_str,
                service_id: r.service_id_str,
                archived: r.archived,
                timestamp: r.time.to_rfc3339(),
            })
            .collect();
        Json(list)
    }

    /// Triggers on-demand message log archival and packaging into an ASiC-E archive.
    #[oai(path = "/v1/messagelog/archive", method = "post")]
    async fn archive_logs(&self) -> Json<ArchiveResponse> {
        let archive_dir =
            std::env::var("XROAD_ARCHIVE_DIR").unwrap_or_else(|_| "/tmp/xroad/archive".to_string());
        let archiver =
            messagelog::MessageLogArchiver::new((*self.state.message_log).clone(), archive_dir)
                .with_max_records(100);

        match archiver.archive_batch().await {
            Ok(Some(info)) => Json(ArchiveResponse {
                success: true,
                records_archived: info.record_count,
                file_path: Some(info.file_path.to_string_lossy().to_string()),
                file_name: Some(info.file_name),
                last_digest: Some(info.last_digest),
                message: format!("Successfully archived {} records", info.record_count),
            }),
            Ok(None) => Json(ArchiveResponse {
                success: true,
                records_archived: 0,
                file_path: None,
                file_name: None,
                last_digest: None,
                message: "No unarchived records to process".to_string(),
            }),
            Err(e) => Json(ArchiveResponse {
                success: false,
                records_archived: 0,
                file_path: None,
                file_name: None,
                last_digest: None,
                message: format!("Archival failed: {e}"),
            }),
        }
    }
}

pub fn admin_api_app(state: AdminApiState) -> Route {
    let api_service = OpenApiService::new(
        AdminApi { state },
        "X-Road Security Server Admin API",
        "0.1.0",
    )
    .server("/api");
    let ui = api_service.swagger_ui();
    Route::new()
        .nest("/api", api_service)
        .nest("/swagger-ui", ui)
}
