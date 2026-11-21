use axum::{Json, Router, routing::get};
use serde::Serialize;

#[derive(Serialize)]
pub struct CentralServerInfo {
    pub instance_identifier: String,
    pub status: String,
}

pub fn central_server_router(instance_id: String) -> Router {
    Router::new().route(
        "/internalconf",
        get(move || async move {
            Json(CentralServerInfo {
                instance_identifier: instance_id,
                status: "READY".to_string(),
            })
        }),
    )
}

/// Launches the Central Server administration and configuration service on the given address.
pub async fn run_central_server(
    bind_addr: &str,
    instance: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Starting X-Road Central Server on {}...", bind_addr);
    let app = central_server_router(instance.to_string());
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
