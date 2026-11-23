pub mod api_keys_api;
pub mod backups_api;
pub mod certification_services_api;
pub mod clients_api;
pub mod configuration_sources_api;
pub mod ds_tls_api;
pub mod dto;
pub mod global_groups_api;
pub mod initialization_api;
pub mod issuer_service_api;
pub mod management_requests_api;
pub mod management_services_api;
pub mod member_classes_api;
pub mod members_api;
pub mod security_servers_api;
pub mod subsystems_api;
pub mod system_api;
pub mod timestamping_services_api;
pub mod tokens_api;
pub mod trusted_anchors_api;
pub mod web_ui;

pub use api_keys_api::ApiKeysApi;
pub use backups_api::BackupsApi;
pub use certification_services_api::CertificationServicesApi;
pub use clients_api::ClientsApi;
pub use configuration_sources_api::ConfigurationSourcesApi;
pub use ds_tls_api::DsTlsApi;
pub use dto::*;
pub use global_groups_api::GlobalGroupsApi;
pub use initialization_api::InitializationApi;
pub use issuer_service_api::IssuerServiceApi;
pub use management_requests_api::ManagementRequestsApi;
pub use management_services_api::ManagementServicesApi;
pub use member_classes_api::MemberClassesApi;
pub use members_api::MembersApi;
pub use security_servers_api::SecurityServersApi;
pub use subsystems_api::SubsystemsApi;
pub use system_api::SystemApi;
pub use timestamping_services_api::TimestampingServicesApi;
pub use tokens_api::TokensApi;
pub use trusted_anchors_api::TrustedAnchorsApi;

use crate::middleware::{init_tracer, CentralServerAuthMiddleware};
use crate::services::CentralServerDb;
use poem::{
    get, handler,
    http::StatusCode,
    middleware::{OpenTelemetryMetrics, OpenTelemetryTracing},
    web::Data,
    EndpointExt, IntoResponse, Response, Route,
};
use poem_openapi::{OpenApiService, Tags};

/// OpenAPI specification tags for categorizing Central Server REST endpoints.
#[derive(Tags)]
pub enum ApiTags {
    /// Initial Central Server setup and instance configuration
    Initialization,
    /// Member classes (e.g. GOV, COM)
    MemberClasses,
    /// X-Road member organizations
    Members,
    /// Member subsystems
    Subsystems,
    /// Global client queries and lookups
    Clients,
    /// Registered Security Servers and authentication certificates
    SecurityServers,
    /// Global groups and member assignments
    GlobalGroups,
    /// Approved Certification Services (CAs)
    CertificationServices,
    /// Approved Timestamping Services (TSAs)
    TimestampingServices,
    /// Configuration sources and anchor generation
    ConfigurationSources,
    /// System parameters, diagnostics, and metrics
    System,
    /// Security Server registration and deletion requests
    ManagementRequests,
    /// Configuration backups and restores
    Backups,
    /// Management API keys and roles
    ApiKeys,
    /// Tokens and Hardware Security Modules
    Tokens,
    /// Trusted anchors management
    TrustedAnchors,
    /// Central Server Management Services configuration
    ManagementServicesConfiguration,
    /// Issuer Service state and lifecycle
    IssuerService,
    /// DataSpace TLS Certification Authorities
    DsTlsCertificationAuthorities,
}

#[derive(Clone)]
pub struct CentralServerState {
    pub instance_id: String,
    pub db: CentralServerDb,
}

pub type AppState = CentralServerState;

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

pub fn central_server_app(instance_id: String, db: CentralServerDb) -> Route {
    let state = CentralServerState {
        instance_id,
        db: db.clone(),
    };

    let tracer = init_tracer("xroad-central-server");
    let auth_middleware = CentralServerAuthMiddleware { db };

    let api_part1 = (
        InitializationApi,
        MemberClassesApi,
        MembersApi,
        SubsystemsApi,
        ClientsApi,
        SecurityServersApi,
        GlobalGroupsApi,
        CertificationServicesApi,
        TimestampingServicesApi,
        ConfigurationSourcesApi,
    );
    let api_part2 = (
        SystemApi,
        ManagementRequestsApi,
        BackupsApi,
        ApiKeysApi,
        TokensApi,
        TrustedAnchorsApi,
        ManagementServicesApi,
        IssuerServiceApi,
        DsTlsApi,
    );
    let api = (api_part1, api_part2);

    let api_service =
        OpenApiService::new(api, "X-Road Central Server Admin API", "0.1.0").server("/api");
    let ui = api_service.swagger_ui();
    let spec_yaml = api_service.spec_endpoint_yaml();
    let spec_yaml_v1 = api_service.spec_endpoint_yaml();
    let spec_json = api_service.spec_endpoint();
    let spec_json_v1 = api_service.spec_endpoint();

    let auth_api = api_service
        .with(auth_middleware)
        .with(OpenTelemetryMetrics::new())
        .with(OpenTelemetryTracing::new(tracer.clone()))
        .data(state.clone())
        .data(tracer);

    Route::new()
        .nest("/api", auth_api)
        .nest("/swagger-ui", ui)
        .at("/openapi.yaml", spec_yaml)
        .at("/api/v1/openapi.yaml", spec_yaml_v1)
        .at("/openapi.json", spec_json)
        .at("/api/v1/openapi.json", spec_json_v1)
        .at(
            "/internalconf/shared-params.xml",
            get(handle_shared_params_xml).data(state.clone()),
        )
        .at(
            "/api/v1/globalconf/shared-params.xml",
            get(handle_shared_params_xml).data(state),
        )
        .nest("/", web_ui::central_server_web_ui())
}
