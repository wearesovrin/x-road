pub mod access_rights_api;
pub mod api_keys_api;
pub mod backups_api;
pub mod certificate_authorities_api;
pub mod clients_api;
pub mod dataspace_api;
pub mod diagnostics_api;
pub mod dto;
pub mod endpoints_api;
pub mod initialization_api;
pub mod keys_api;
pub mod local_groups_api;
pub mod member_classes_api;
pub mod message_log_api;
pub mod services_api;
pub mod system_api;
pub mod token_certificates_api;
pub mod tokens_api;
pub mod web_ui;

pub use access_rights_api::AccessRightsApi;
pub use api_keys_api::ApiKeysApi;
pub use backups_api::BackupsApi;
pub use certificate_authorities_api::CertificateAuthoritiesApi;
pub use clients_api::ClientsApi;
pub use dataspace_api::DataspaceApi;
pub use diagnostics_api::DiagnosticsApi;
pub use dto::*;
pub use endpoints_api::EndpointsApi;
pub use initialization_api::InitializationApi;
pub use keys_api::KeysApi;
pub use local_groups_api::LocalGroupsApi;
pub use member_classes_api::MemberClassesApi;
pub use message_log_api::MessageLogApi;
pub use services_api::ServicesApi;
pub use system_api::SystemApi;
pub use token_certificates_api::TokenCertificatesApi;
pub use tokens_api::TokensApi;

use crate::middleware::{init_tracer, SecurityServerAuthMiddleware};
use globalconf::GlobalConf;
use messagelog::MessageLogDb;
use poem::{
    middleware::{OpenTelemetryMetrics, OpenTelemetryTracing},
    EndpointExt, Route,
};
use poem_openapi::{
    param::Path,
    payload::{Json, PlainText},
    OpenApiService, Tags,
};

/// OpenAPI specification tags for categorizing Security Server REST endpoints.
#[derive(Tags)]
pub enum ApiTags {
    /// Initial Security Server setup and configuration
    Initialization,
    /// System diagnostics, parameters, and environmental metrics
    System,
    /// Diagnostics endpoints
    Diagnostics,
    /// Client management (members and subsystems)
    Clients,
    /// Services, REST endpoints, and service descriptions
    Services,
    /// Service endpoint definitions
    Endpoints,
    /// Local client access groups
    LocalGroups,
    /// Service access rights management
    AccessRights,
    /// Cryptographic hardware/software tokens
    Tokens,
    /// Cryptographic key management
    Keys,
    /// Token certificates management
    TokenCertificates,
    /// Approved Certification Authorities (CAs)
    CertificateAuthorities,
    /// Dataspace provisioning and agreement keys
    Dataspace,
    /// Member classes and names
    MemberClasses,
    /// System configuration backups and restores
    Backups,
    /// Management API keys and roles
    ApiKeys,
    /// Operational message logging and audit trails
    MessageLog,
}
use serverconf::DatabaseServerConf;
use signer::SignerServiceImpl;
use std::sync::Arc;
use tokio::sync::RwLock;

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

pub type AppState = AdminApiState;

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

pub struct AdminApi {
    pub state: AdminApiState,
}

impl AdminApi {
    pub fn new(state: AdminApiState) -> Self {
        Self { state }
    }

    pub async fn get_status(&self) -> Json<ServerStatus> {
        SystemApi.get_status(poem::web::Data(&self.state)).await
    }

    pub async fn get_metrics(&self) -> PlainText<String> {
        SystemApi.get_metrics().await
    }

    pub async fn create_api_key(&self, req: Json<CreateApiKeyReq>) -> Json<ApiKeyResponse> {
        ApiKeysApi
            .create_api_key(poem::web::Data(&self.state), req)
            .await
    }

    pub async fn list_api_keys(&self) -> Json<Vec<ApiKeyDto>> {
        ApiKeysApi.list_api_keys(poem::web::Data(&self.state)).await
    }

    pub async fn get_api_key(&self, id: Path<i64>) -> poem::Result<Json<ApiKeyDto>> {
        ApiKeysApi
            .get_api_key(poem::web::Data(&self.state), id)
            .await
    }

    pub async fn update_api_key(
        &self,
        id: Path<i64>,
        req: Json<UpdateApiKeyReq>,
    ) -> poem::Result<Json<ApiKeyDto>> {
        ApiKeysApi
            .update_api_key(poem::web::Data(&self.state), id, req)
            .await
    }

    pub async fn revoke_api_key(&self, id: Path<i64>) -> Json<bool> {
        ApiKeysApi
            .revoke_api_key(poem::web::Data(&self.state), id)
            .await
    }

    pub async fn validate_api_key(
        &self,
        req: Json<ValidateApiKeyReq>,
    ) -> Json<ValidateApiKeyResponse> {
        ApiKeysApi
            .validate_api_key(poem::web::Data(&self.state), req)
            .await
    }
}

pub fn admin_api_app(state: AdminApiState) -> Route {
    let auth_middleware = SecurityServerAuthMiddleware {
        server_conf: state.server_conf.clone(),
    };

    let api_part1 = (
        InitializationApi,
        SystemApi,
        DiagnosticsApi,
        ClientsApi,
        ServicesApi,
        EndpointsApi,
        LocalGroupsApi,
        AccessRightsApi,
        TokensApi,
        KeysApi,
    );

    let api_part2 = (
        TokenCertificatesApi,
        CertificateAuthoritiesApi,
        DataspaceApi,
        MemberClassesApi,
        BackupsApi,
        ApiKeysApi,
        MessageLogApi,
    );

    let api = (api_part1, api_part2);

    let api_service =
        OpenApiService::new(api, "X-Road Security Server Admin API", "0.1.0").server("/api");
    let ui = api_service.swagger_ui();
    let spec_yaml = api_service.spec_endpoint_yaml();
    let spec_yaml_v1 = api_service.spec_endpoint_yaml();
    let spec_json = api_service.spec_endpoint();
    let spec_json_v1 = api_service.spec_endpoint();
    let tracer = init_tracer("xroad-security-server");
    let auth_api = api_service
        .with(auth_middleware)
        .with(OpenTelemetryMetrics::new())
        .with(OpenTelemetryTracing::new(tracer.clone()))
        .data(state)
        .data(tracer);
    Route::new()
        .nest("/api", auth_api)
        .nest("/swagger-ui", ui)
        .at("/openapi.yaml", spec_yaml)
        .at("/api/v1/openapi.yaml", spec_yaml_v1)
        .at("/openapi.json", spec_json)
        .at("/api/v1/openapi.json", spec_json_v1)
        .nest("/", web_ui::security_server_web_ui())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_server_admin_api_keys() {
        let sc = Arc::new(DatabaseServerConf::connect_in_memory().await.unwrap());
        let mlog = Arc::new(MessageLogDb::connect_in_memory().await.unwrap());
        let gconf = Arc::new(GlobalConf::default());
        let signer = Arc::new(SignerServiceImpl::new());

        let state = AdminApiState::new(sc.clone(), mlog, gconf, signer);
        let api = AdminApi {
            state: state.clone(),
        };

        // 1. Check status & metrics
        let status = api.get_status().await;
        assert_eq!(status.0.status, "OK");

        let metrics = api.get_metrics().await;
        assert!(metrics.0.contains("xroad_system_cpu_load"));

        // 2. Create API key
        let create_req = CreateApiKeyReq {
            key: None,
            roles: vec!["XROAD_SECURITY_OFFICER".to_string()],
            label: Some("test-key".to_string()),
        };
        let key = api.create_api_key(Json(create_req)).await;
        assert_eq!(key.0.roles, vec!["XROAD_SECURITY_OFFICER"]);

        // 3. Validate API key
        let val_req = ValidateApiKeyReq {
            key: key.0.key.clone(),
        };
        let val_res = api.validate_api_key(Json(val_req)).await;
        assert!(val_res.0.valid);
        assert_eq!(val_res.0.roles, vec!["XROAD_SECURITY_OFFICER"]);

        // 4. Revoke API key
        let revoked = api.revoke_api_key(Path(key.0.id)).await;
        assert!(revoked.0);
    }
}
