use poem::{Route, Server, listener::TcpListener};
use poem_openapi::{Object, OpenApi, OpenApiService, payload::Json};

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct CentralServerInfo {
    pub instance_identifier: String,
    pub status: String,
}

pub struct CentralServerApi {
    pub instance_id: String,
}

#[OpenApi]
impl CentralServerApi {
    /// Returns the Central Server operational status and configuration endpoint.
    #[oai(path = "/internalconf", method = "get")]
    async fn internal_conf(&self) -> Json<CentralServerInfo> {
        Json(CentralServerInfo {
            instance_identifier: self.instance_id.clone(),
            status: "READY".to_string(),
        })
    }
}

pub fn central_server_app(instance_id: String) -> Route {
    let api_service = OpenApiService::new(
        CentralServerApi { instance_id },
        "X-Road Central Server Admin API",
        "0.1.0",
    )
    .server("/api");
    let ui = api_service.swagger_ui();
    Route::new()
        .nest("/api", api_service)
        .nest("/swagger-ui", ui)
}

/// Launches the Central Server administration and configuration service on the given address.
pub async fn run_central_server(
    bind_addr: &str,
    instance: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Starting X-Road Central Server on {} (Swagger UI at /swagger-ui)...",
        bind_addr
    );
    let app = central_server_app(instance.to_string());
    Server::new(TcpListener::bind(bind_addr)).run(app).await?;
    Ok(())
}
