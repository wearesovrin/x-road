use common::{XRoadError, XRoadResult};
use globalconf::{ConfigurationAnchor, GlobalConf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Configuration client daemon responsible for downloading, verifying, and caching global configuration.
#[derive(Debug, Clone)]
pub struct ConfClient {
    pub anchor: ConfigurationAnchor,
    pub cached_conf: Arc<RwLock<Option<GlobalConf>>>,
    pub http_client: reqwest::Client,
}

impl ConfClient {
    pub fn new(anchor: ConfigurationAnchor) -> Self {
        Self {
            anchor,
            cached_conf: Arc::new(RwLock::new(None)),
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Fetches the latest global configuration from the Central Server anchor download URLs.
    pub async fn sync_once(&self) -> XRoadResult<GlobalConf> {
        let mut last_err = None;

        for url in &self.anchor.download_urls {
            tracing::debug!("Attempting globalconf download from: {}", url);
            match self.http_client.get(url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let xml = resp.text().await.map_err(|e| {
                        XRoadError::Network(format!("Failed to read globalconf body from {url}: {e}"))
                    })?;

                    let conf = GlobalConf::from_shared_params_xml(&xml)?;
                    if conf.instance_identifier != self.anchor.instance_identifier {
                        return Err(XRoadError::Configuration(format!(
                            "Instance identifier mismatch: expected {}, got {}",
                            self.anchor.instance_identifier, conf.instance_identifier
                        )));
                    }

                    *self.cached_conf.write().await = Some(conf.clone());
                    tracing::info!(
                        "Successfully updated global configuration for instance: {} ({} members, {} security servers)",
                        conf.instance_identifier,
                        conf.members.len(),
                        conf.security_servers.len()
                    );
                    return Ok(conf);
                }
                Ok(resp) => {
                    let err = format!("Server returned HTTP {}", resp.status());
                    tracing::warn!("Failed to fetch globalconf from {}: {}", url, err);
                    last_err = Some(err);
                }
                Err(e) => {
                    let err = e.to_string();
                    tracing::warn!("Network error connecting to {}: {}", url, err);
                    last_err = Some(err);
                }
            }
        }

        Err(XRoadError::Configuration(format!(
            "Failed to synchronize global configuration from any download URL: {:?}",
            last_err
        )))
    }

    /// Returns the currently cached `GlobalConf`, if any.
    pub async fn get_global_conf(&self) -> Option<GlobalConf> {
        self.cached_conf.read().await.clone()
    }

    /// Spawns a background synchronization loop that periodically updates the configuration.
    pub fn start_sync_worker(self: Arc<Self>, interval: Duration) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                if let Err(e) = self.sync_once().await {
                    tracing::error!("Background global configuration sync failed: {}", e);
                }
            }
        })
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

    let client = Arc::new(ConfClient::new(anchor));
    client.sync_once().await?;
    let _worker = client.clone().start_sync_worker(Duration::from_secs(60));

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Stopping X-Road Configuration Client Daemon...");
    Ok(())
}
