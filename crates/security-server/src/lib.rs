pub mod admin_api;
pub mod config;
pub mod node;

pub use admin_api::{admin_api_app, AdminApi, AdminApiState, ServerStatus};
pub use config::SecurityServerConfig;
pub use node::SecurityServerNode;

use poem::listener::TcpListener;
use poem::Server;
use serverconf::DatabaseServerConf;

/// Backward-compatible helper to run just the Admin REST API.
pub async fn run_admin_api(
    bind_addr: &str,
    db_url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server_conf = match db_url {
        Some(url) => DatabaseServerConf::connect(url).await?,
        None => DatabaseServerConf::connect_in_memory().await?,
    };
    let message_log = messagelog::MessageLogDb::connect_in_memory().await?;
    let state = AdminApiState::new(
        std::sync::Arc::new(server_conf),
        std::sync::Arc::new(message_log),
        std::sync::Arc::new(globalconf::GlobalConf::new("DEV")),
        std::sync::Arc::new(signer::SignerServiceImpl::new()),
    );

    tracing::info!(
        "Starting X-Road Security Server Admin REST API on {} (Swagger UI at /swagger-ui)...",
        bind_addr
    );
    let app = admin_api_app(state);
    Server::new(TcpListener::bind(bind_addr)).run(app).await?;
    Ok(())
}

/// Launches the complete, unified X-Road Security Server (Admin API, Client Proxy, Server Proxy).
pub async fn run_security_server(
    config: SecurityServerConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let node = SecurityServerNode::connect_from_config(config).await?;
    node.run().await
}
