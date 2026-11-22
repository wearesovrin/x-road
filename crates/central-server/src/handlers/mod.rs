pub mod api_keys_api;
pub mod backups_api;
pub mod certification_services_api;
pub mod clients_api;
pub mod configuration_sources_api;
pub mod dto;
pub mod global_groups_api;
pub mod initialization_api;
pub mod management_requests_api;
pub mod member_classes_api;
pub mod members_api;
pub mod security_servers_api;
pub mod subsystems_api;
pub mod system_api;
pub mod timestamping_services_api;

pub use api_keys_api::ApiKeysApi;
pub use backups_api::BackupsApi;
pub use certification_services_api::CertificationServicesApi;
pub use clients_api::ClientsApi;
pub use configuration_sources_api::ConfigurationSourcesApi;
pub use dto::*;
pub use global_groups_api::GlobalGroupsApi;
pub use initialization_api::InitializationApi;
pub use management_requests_api::ManagementRequestsApi;
pub use member_classes_api::MemberClassesApi;
pub use members_api::MembersApi;
pub use security_servers_api::SecurityServersApi;
pub use subsystems_api::SubsystemsApi;
pub use system_api::SystemApi;
pub use timestamping_services_api::TimestampingServicesApi;

use crate::services::CentralServerDb;
use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{SdkTracer, SdkTracerProvider},
};
use poem::{
    EndpointExt, IntoResponse, Response, Route, get, handler,
    http::StatusCode,
    middleware::{OpenTelemetryMetrics, OpenTelemetryTracing},
    web::Data,
};
use poem_openapi::OpenApiService;

pub fn init_tracer(service_name: &str) -> SdkTracer {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let mut builder = SdkTracerProvider::builder().with_resource(
        Resource::builder()
            .with_service_name(service_name.to_string())
            .build(),
    );

    if let Ok(exporter) = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
    {
        builder = builder.with_batch_exporter(exporter);
    }

    let provider = builder.build();
    let tracer = provider.tracer(service_name.to_string());
    global::set_tracer_provider(provider);
    tracer
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
        instance_id,
        db: db.clone(),
    };

    let tracer = init_tracer("xroad-central-server");
    let auth_middleware = CentralServerAuthMiddleware { db };

    let api = (
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
        SystemApi,
        ManagementRequestsApi,
        BackupsApi,
        ApiKeysApi,
    );

    let api_service =
        OpenApiService::new(api, "X-Road Central Server Admin API", "0.1.0").server("/api");
    let ui = api_service.swagger_ui();
    let auth_api = api_service
        .with(auth_middleware)
        .with(OpenTelemetryMetrics::new())
        .with(OpenTelemetryTracing::new(tracer.clone()))
        .data(state.clone())
        .data(tracer);

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
