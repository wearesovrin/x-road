use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "central-server")]
#[command(about = "X-Road Central Server Management & Global Configuration Daemon")]
struct Cli {
    /// Bind address for Central Server Admin API
    #[arg(short, long, env = "BIND_ADDR", default_value = "0.0.0.0:4001")]
    bind: String,

    /// X-Road Instance identifier (e.g. DEV, PROD)
    #[arg(short, long, env = "XROAD_INSTANCE", default_value = "DEV")]
    instance: String,

    /// Central Server SQLite or PostgreSQL database URL
    #[arg(short, long, env = "DATABASE_URL")]
    database_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    central_server::run_central_server(&cli.bind, &cli.instance, cli.database_url.as_deref()).await
}
