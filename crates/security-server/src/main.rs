use clap::Parser;
use security_server::{SecurityServerConfig, SecurityServerNode};

#[derive(Parser, Debug)]
#[command(name = "security-server")]
#[command(
    about = "Unified X-Road Security Server Node (Admin API, Client Proxy, Server Proxy, mTLS)"
)]
struct Cli {
    /// Bind address for Security Server Admin API
    #[arg(long, env = "ADMIN_BIND_ADDR", default_value = "0.0.0.0:4000")]
    admin_bind: String,

    /// Bind address for Client Proxy (outgoing consumer gateway)
    #[arg(long, env = "CLIENT_PROXY_BIND_ADDR", default_value = "0.0.0.0:8000")]
    client_proxy_bind: String,

    /// Bind address for Server Proxy (incoming provider gateway)
    #[arg(long, env = "SERVER_PROXY_BIND_ADDR", default_value = "0.0.0.0:8443")]
    server_proxy_bind: String,

    /// Bind address for native mTLS listener (X-Road Wire Port 5500)
    #[arg(long, env = "MTLS_BIND_ADDR", default_value = "0.0.0.0:5500")]
    mtls_bind: String,

    /// Enable native mTLS socket binding on TCP port 5500
    #[arg(long, env = "XROAD_ENABLE_MTLS")]
    enable_mtls: bool,

    /// ServerConf SQLite or PostgreSQL database URL
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,

    /// MessageLog SQLite or PostgreSQL database URL
    #[arg(long, env = "MESSAGELOG_DATABASE_URL")]
    messagelog_database_url: Option<String>,

    /// Default target server proxy URL fallback
    #[arg(long, env = "DEFAULT_SERVER_PROXY_URL")]
    default_server_proxy_url: Option<String>,

    /// Path to static Vue.js Admin UI build directory (dist)
    #[arg(long, env = "XROAD_UI_PATH")]
    ui_static_path: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let cli = Cli::parse();
    let config = SecurityServerConfig {
        admin_bind: cli.admin_bind,
        client_proxy_bind: cli.client_proxy_bind,
        server_proxy_bind: cli.server_proxy_bind,
        mtls_bind: cli.mtls_bind,
        enable_mtls: cli.enable_mtls,
        database_url: cli.database_url,
        messagelog_database_url: cli.messagelog_database_url,
        default_server_proxy_url: cli.default_server_proxy_url,
        ui_static_path: cli.ui_static_path,
    };
    let node = SecurityServerNode::connect_from_config(config).await?;
    node.run().await
}
