#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:4001".to_string());
    let instance = std::env::var("XROAD_INSTANCE").unwrap_or_else(|_| "DEV".to_string());
    let db_url = std::env::var("DATABASE_URL").ok();
    central_server::run_central_server(&bind_addr, &instance, db_url.as_deref()).await
}
