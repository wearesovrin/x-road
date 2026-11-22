pub mod access_rights_api;
pub mod api_keys_api;
pub mod backups_api;
pub mod certificate_authorities_api;
pub mod clients_api;
pub mod dto;
pub mod endpoints_api;
pub mod initialization_api;
pub mod local_groups_api;
pub mod message_log_api;
pub mod services_api;
pub mod system_api;
pub mod tokens_api;
pub mod web_ui;

pub use access_rights_api::AccessRightsApi;
pub use api_keys_api::ApiKeysApi;
pub use backups_api::BackupsApi;
pub use certificate_authorities_api::CertificateAuthoritiesApi;
pub use clients_api::ClientsApi;
pub use dto::*;
pub use endpoints_api::EndpointsApi;
pub use initialization_api::InitializationApi;
pub use local_groups_api::LocalGroupsApi;
pub use message_log_api::MessageLogApi;
pub use services_api::ServicesApi;
pub use system_api::SystemApi;
pub use tokens_api::TokensApi;

use crate::middleware::{SecurityServerAuthMiddleware, init_tracer};
use globalconf::GlobalConf;
use messagelog::MessageLogDb;
use poem::{
    EndpointExt, Route,
    middleware::{OpenTelemetryMetrics, OpenTelemetryTracing},
};
use poem_openapi::{
    OpenApiService,
    param::Path,
    payload::{Json, PlainText},
};
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

// -----------------------------------------------------------------------------
// Compatibility Wrapper: AdminApi
// -----------------------------------------------------------------------------

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

    let api = (
        InitializationApi,
        SystemApi,
        ClientsApi,
        ServicesApi,
        EndpointsApi,
        LocalGroupsApi,
        AccessRightsApi,
        TokensApi,
        CertificateAuthoritiesApi,
        BackupsApi,
        ApiKeysApi,
        MessageLogApi,
    );

    let api_service =
        OpenApiService::new(api, "X-Road Security Server Admin API", "0.1.0").server("/api");
    let ui = api_service.swagger_ui();
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
            label: Some("Test Key".to_string()),
            roles: vec!["XROAD_SECURITY_OFFICER".to_string()],
        };
        let created = api.create_api_key(Json(create_req)).await.0;
        assert!(!created.key_id.is_empty());
        assert!(created.key.is_some());
        let plaintext_token = created.key.unwrap();

        // 3. List API keys
        let list = api.list_api_keys().await.0;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);

        // 4. Validate API key using plaintext token
        let val_req = ValidateApiKeyReq {
            key: plaintext_token.clone(),
        };
        let val_resp = api.validate_api_key(Json(val_req)).await.0;
        assert!(val_resp.valid);
        assert_eq!(val_resp.roles, vec!["XROAD_SECURITY_OFFICER".to_string()]);

        // 5. Update roles
        let update_req = UpdateApiKeyReq {
            roles: vec!["XROAD_SYSTEM_ADMINISTRATOR".to_string()],
        };
        let updated = api
            .update_api_key(Path(created.id), Json(update_req))
            .await
            .unwrap()
            .0;
        assert_eq!(
            updated.roles,
            vec!["XROAD_SYSTEM_ADMINISTRATOR".to_string()]
        );

        // 6. Revoke API key
        let revoked = api.revoke_api_key(Path(created.id)).await.0;
        assert!(revoked);

        let list_after = api.list_api_keys().await.0;
        assert!(list_after.is_empty());
    }
}
