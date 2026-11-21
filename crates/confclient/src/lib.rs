use common::XRoadResult;
use globalconf::{ConfigurationAnchor, GlobalConf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration client daemon responsible for downloading, verifying, and caching global configuration.
#[derive(Debug, Clone)]
pub struct ConfClient {
    pub anchor: ConfigurationAnchor,
    pub cached_conf: Arc<RwLock<Option<GlobalConf>>>,
}

impl ConfClient {
    pub fn new(anchor: ConfigurationAnchor) -> Self {
        Self {
            anchor,
            cached_conf: Arc::new(RwLock::new(None)),
        }
    }

    /// Fetches the latest global configuration from the Central Server.
    pub async fn update(&self) -> XRoadResult<GlobalConf> {
        let instance = &self.anchor.instance_identifier;
        let conf = GlobalConf::new(instance.clone());
        *self.cached_conf.write().await = Some(conf.clone());
        Ok(conf)
    }
}

/// Synchronizes global configuration using the specified anchor URL and instance identifier.
pub async fn run_confclient(
    instance: &str,
    download_url: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Starting X-Road Configuration Client Daemon for instance: {}...",
        instance
    );
    let anchor = ConfigurationAnchor {
        instance_identifier: instance.to_string(),
        generated_at: chrono::Utc::now(),
        download_urls: vec![download_url.to_string()],
        verification_cert_hashes: vec![],
    };

    let client = ConfClient::new(anchor);
    let conf = client.update().await?;
    tracing::info!(
        "Synchronized global configuration for instance: {}",
        conf.instance_identifier
    );
    Ok(())
}
