use clap::{Parser, Subcommand};
use identifiers::ClientId;

#[derive(Parser, Debug)]
#[command(name = "x-road")]
#[command(about = "Unified X-Road Node & Daemon Launcher", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the Security Server Proxy daemon
    Proxy {
        #[arg(short, long, default_value = "127.0.0.1:8080")]
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
    /// Start the Security Server REST administration API
    Admin {
        #[arg(short, long, default_value = "127.0.0.1:4000")]
        bind: String,
    },
    /// Start the Central Server daemon
    Central {
        #[arg(short, long, default_value = "127.0.0.1:8000")]
        bind: String,
        #[arg(short, long, default_value = "DEV")]
        instance: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Proxy { bind, client } => {
            let client_id = client
                .parse::<ClientId>()
                .unwrap_or_else(|_| ClientId::member("DEV", "COM", "12345678"));
            proxy::run_client_proxy(&bind, client_id).await?;
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
        Commands::Admin { bind } => {
            security_server::run_admin_api(&bind).await?;
        }
        Commands::Central { bind, instance } => {
            central_server::run_central_server(&bind, &instance).await?;
        }
    }

    Ok(())
}
