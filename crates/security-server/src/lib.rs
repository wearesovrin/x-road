use poem::{Route, Server, listener::TcpListener};
use poem_openapi::{Object, OpenApi, OpenApiService, payload::Json};
use serverconf::DatabaseServerConf;

#[derive(Object, serde::Serialize, Debug, Clone)]
pub struct ServerStatus {
    pub status: String,
    pub version: String,
    pub client_count: usize,
}

pub struct Api {
    pub db_conf: DatabaseServerConf,
}

#[OpenApi]
impl Api {
    /// Returns the current operational status and version of the Security Server.
    #[oai(path = "/v1/status", method = "get")]
    async fn get_status(&self) -> Json<ServerStatus> {
        let clients = self.db_conf.list_clients().await.unwrap_or_default();
        Json(ServerStatus {
            status: "OK".to_string(),
            version: "0.1.0".to_string(),
            client_count: clients.len(),
        })
    }
}

pub fn security_server_app(db_conf: DatabaseServerConf) -> Route {
    let api_service =
        OpenApiService::new(Api { db_conf }, "X-Road Security Server Admin API", "0.1.0").server("/api");
    let ui = api_service.swagger_ui();
    Route::new()
        .nest("/api", api_service)
        .nest("/swagger-ui", ui)
}

/// Launches the Security Server Admin REST API on the given address.
pub async fn run_admin_api(
    bind_addr: &str,
    db_url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_conf = match db_url {
        Some(url) => DatabaseServerConf::connect(url).await?,
        None => DatabaseServerConf::connect_in_memory().await?,
    };

    tracing::info!(
        "Starting X-Road Security Server Admin REST API on {} (Swagger UI at /swagger-ui)...",
        bind_addr
    );
    let app = security_server_app(db_conf);
    Server::new(TcpListener::bind(bind_addr)).run(app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_server_app_in_memory() {
        let db_conf = DatabaseServerConf::connect_in_memory().await.unwrap();
        let _app = security_server_app(db_conf);
    }
}
