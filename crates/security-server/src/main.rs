#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:4000".to_string());
    let db_url = std::env::var("DATABASE_URL").ok();
    security_server::run_admin_api(&bind_addr, db_url.as_deref()).await
}
