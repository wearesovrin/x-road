use poem::{Route, Server, listener::TcpListener};
use poem_openapi::{Object, OpenApi, OpenApiService, payload::Json};

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ServerStatus {
    pub status: String,
    pub version: String,
}

pub struct Api;

#[OpenApi]
impl Api {
    /// Returns the current operational status and version of the Security Server.
    #[oai(path = "/v1/status", method = "get")]
    async fn get_status(&self) -> Json<ServerStatus> {
        Json(ServerStatus {
            status: "OK".to_string(),
            version: "0.1.0".to_string(),
        })
    }
}

pub fn security_server_app() -> Route {
    let api_service =
        OpenApiService::new(Api, "X-Road Security Server Admin API", "0.1.0").server("/api");
    let ui = api_service.swagger_ui();
    Route::new()
        .nest("/api", api_service)
        .nest("/swagger-ui", ui)
}

/// Launches the Security Server Admin REST API on the given address.
pub async fn run_admin_api(
    bind_addr: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Starting X-Road Security Server Admin REST API on {} (Swagger UI at /swagger-ui)...",
        bind_addr
    );
    let app = security_server_app();
    Server::new(TcpListener::bind(bind_addr)).run(app).await?;
    Ok(())
}
