use clap::{Parser, Subcommand};
use identifiers::ClientId;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "x-road")]
#[command(about = "Unified X-Road Node & Daemon Launcher", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the complete unified Security Server node (Admin API, Client Proxy, Server Proxy)
    #[command(name = "security-server", alias = "admin")]
    SecurityServer {
        #[arg(long, env = "ADMIN_BIND_ADDR", default_value = "0.0.0.0:4000")]
        admin_bind: String,
        #[arg(long, env = "CLIENT_PROXY_BIND_ADDR", default_value = "0.0.0.0:8000")]
        client_proxy_bind: String,
        #[arg(long, env = "SERVER_PROXY_BIND_ADDR", default_value = "0.0.0.0:8443")]
        server_proxy_bind: String,
        #[arg(long, env = "DATABASE_URL")]
        database_url: Option<String>,
        #[arg(long, env = "MESSAGELOG_DATABASE_URL")]
        messagelog_database_url: Option<String>,
        #[arg(long, env = "DEFAULT_SERVER_PROXY_URL")]
        default_server_proxy_url: Option<String>,
    },
    /// Start the Central Server daemon (OpenAPI Management & Global Configuration distribution)
    #[command(name = "central-server", alias = "central")]
    CentralServer {
        #[arg(short, long, env = "BIND_ADDR", default_value = "0.0.0.0:4001")]
        bind: String,
        #[arg(short, long, env = "XROAD_INSTANCE", default_value = "DEV")]
        instance: String,
        #[arg(long, env = "DATABASE_URL")]
        database_url: Option<String>,
    },
    /// Start the standalone Security Server Client Proxy daemon
    Proxy {
        #[arg(short, long, default_value = "127.0.0.1:8000")]
        bind: String,
        #[arg(short, long, default_value = "DEV/COM/12345678")]
        client: String,
    },
    /// Start the Signer key management daemon
    Signer {
        #[arg(short, long, default_value = "127.0.0.1:50051")]
        bind: String,
    },
    /// Start the Configuration Client synchronization daemon
    Confclient {
        #[arg(short, long, default_value = "DEV")]
        instance: String,
        #[arg(short, long, default_value = "http://central-server/internalconf")]
        url: String,
    },
    /// Start the Environmental Monitoring daemon
    Monitor {
        #[arg(short, long, default_value = "127.0.0.1:50052")]
        bind: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::SecurityServer {
            admin_bind,
            client_proxy_bind,
            server_proxy_bind,
            database_url,
            messagelog_database_url,
            default_server_proxy_url,
        } => {
            let config = security_server::SecurityServerConfig {
                admin_bind,
                client_proxy_bind,
                server_proxy_bind,
                database_url,
                messagelog_database_url,
                default_server_proxy_url,
            };
            security_server::run_security_server(config).await?;
        }
        Commands::CentralServer {
            bind,
            instance,
            database_url,
        } => {
            let db_url = database_url.or_else(|| std::env::var("DATABASE_URL").ok());
            central_server::run_central_server(&bind, &instance, db_url.as_deref()).await?;
        }
        Commands::Proxy { bind, client } => {
            let client_id = client
                .parse::<ClientId>()
                .unwrap_or_else(|_| ClientId::member_unchecked("DEV", "COM", "12345678"));
            let db_url = std::env::var("DATABASE_URL").ok();
            let server_conf = match db_url.as_deref() {
                Some(url) => Arc::new(serverconf::DatabaseServerConf::connect(url).await?),
                None => Arc::new(serverconf::DatabaseServerConf::connect_in_memory().await?),
            };
            let _ = server_conf.add_client(&client_id, false).await;
            let message_log = Arc::new(messagelog::MessageLogDb::connect_in_memory().await?);
            let global_conf = Arc::new(globalconf::GlobalConf::new("DEV"));
            let state = proxy::ClientProxyState::new(server_conf, message_log, global_conf);
            proxy::run_client_proxy(&bind, state).await?;
        }
        Commands::Signer { bind } => {
            signer::run_signer(&bind).await?;
        }
        Commands::Confclient { instance, url } => {
            confclient::run_confclient(&instance, &url).await?;
        }
        Commands::Monitor { bind } => {
            monitor::run_monitor(&bind).await?;
        }
    }

    Ok(())
}
