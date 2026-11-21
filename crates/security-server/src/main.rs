use security_server::{SecurityServerConfig, SecurityServerNode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    let config = SecurityServerConfig::default();
    let node = SecurityServerNode::connect_from_config(config).await?;
    node.run().await
}
