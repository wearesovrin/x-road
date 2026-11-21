use base64::Engine;
use identifiers::{ClientId, ServiceId};
use messagelog::MessageLogDb;
use poem::Route;
use poem_openapi::{payload::Json, Object, OpenApi, OpenApiService};
use serverconf::{DatabaseServerConf, ServiceType};
use std::sync::Arc;

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ServerStatus {
    pub status: String,
    pub version: String,
    pub client_count: usize,
    pub unarchived_logs: usize,
}

#[derive(Object, serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ClientResponse {
    pub id: String,
    pub status: String,
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

#[derive(Clone)]
pub struct AdminApiState {
    pub server_conf: Arc<DatabaseServerConf>,
    pub message_log: Arc<MessageLogDb>,
}

pub struct AdminApi {
    pub state: AdminApiState,
}

#[OpenApi]
impl AdminApi {
    /// Returns the current operational status and version of the Security Server.
    #[oai(path = "/v1/status", method = "get")]
    async fn get_status(&self) -> Json<ServerStatus> {
        let clients = self.state.server_conf.list_clients().await.unwrap_or_default();
        let logs = self.state.message_log.find_unarchived(100).await.unwrap_or_default();

        Json(ServerStatus {
            status: "OK".to_string(),
            version: "0.1.0".to_string(),
            client_count: clients.len(),
            unarchived_logs: logs.len(),
        })
    }

    /// Lists all registered clients.
    #[oai(path = "/v1/clients", method = "get")]
    async fn list_clients(&self) -> Json<Vec<ClientResponse>> {
        let clients = self.state.server_conf.list_clients().await.unwrap_or_default();
        let mut list = Vec::new();
        for c in clients {
            if let Ok(Some(model)) = self.state.server_conf.get_client(&c).await {
                list.push(ClientResponse {
                    id: c.to_string(),
                    status: model.client_status,
                    is_auth: model.is_authentication,
                });
            }
        }
        Json(list)
    }

    /// Registers a client on this Security Server.
    #[oai(path = "/v1/clients", method = "post")]
    async fn register_client(&self, req: Json<RegisterClientReq>) -> Json<SimpleSuccessResponse> {
        let client_id = if let Some(ref sub) = req.subsystem_code {
            ClientId::subsystem_unchecked(&req.instance, &req.member_class, &req.member_code, sub)
        } else {
            ClientId::member_unchecked(&req.instance, &req.member_class, &req.member_code)
        };

        match self
            .state
            .server_conf
            .add_client(&client_id, req.is_authentication.unwrap_or(false))
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

    /// Registers a service for a client.
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

        match self.state.server_conf.grant_access(&service_id, &client_id).await {
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

        match self.state.server_conf.revoke_access(&service_id, &client_id).await {
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

    /// Creates an API key.
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
            Ok(Some(roles)) => Json(ValidateApiKeyResponse {
                valid: true,
                roles,
            }),
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

        let der_bytes = match base64::engine::general_purpose::STANDARD.decode(&req.cert_der_base64) {
            Ok(b) => b,
            Err(e) => {
                return Json(SimpleSuccessResponse {
                    success: false,
                    message: format!("Invalid base64 encoding: {e}"),
                });
            }
        };

        match self.state.server_conf.store_certificate(&client_id, &der_bytes).await {
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
        let records = self.state.message_log.find_unarchived(50).await.unwrap_or_default();
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
}

pub fn admin_api_app(state: AdminApiState) -> Route {
    let api_service =
        OpenApiService::new(AdminApi { state }, "X-Road Security Server Admin API", "0.1.0")
            .server("/api");
    let ui = api_service.swagger_ui();
    Route::new()
        .nest("/api", api_service)
        .nest("/swagger-ui", ui)
}
