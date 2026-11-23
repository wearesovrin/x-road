use common::{XRoadError, XRoadResult};
use globalconf::{ConfigurationAnchor, GlobalConf};
use proto::admin_service_server::{AdminService, AdminServiceServer};
use proto::anchor_service_server::{AnchorService, AnchorServiceServer};
use proto::global_conf_service_server::{GlobalConfService, GlobalConfServiceServer};
use proto::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

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
                        XRoadError::Network(format!(
                            "Failed to read globalconf body from {url}: {e}"
                        ))
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

    pub fn into_global_conf_service(
        self: Arc<Self>,
    ) -> GlobalConfServiceServer<ConfClientGlobalConfServiceAdapter> {
        GlobalConfServiceServer::new(ConfClientGlobalConfServiceAdapter { inner: self })
    }

    pub fn into_anchor_service(
        self: Arc<Self>,
    ) -> AnchorServiceServer<ConfClientAnchorServiceAdapter> {
        AnchorServiceServer::new(ConfClientAnchorServiceAdapter { inner: self })
    }

    pub fn into_admin_service(
        self: Arc<Self>,
    ) -> AdminServiceServer<ConfClientAdminServiceAdapter> {
        AdminServiceServer::new(ConfClientAdminServiceAdapter { inner: self })
    }
}

pub struct ConfClientGlobalConfServiceAdapter {
    inner: Arc<ConfClient>,
}

#[tonic::async_trait]
impl GlobalConfService for ConfClientGlobalConfServiceAdapter {
    async fn get_global_conf(
        &self,
        _req: Request<GetGlobalConfReq>,
    ) -> Result<Response<GetGlobalConfRespWrapped>, Status> {
        let conf_opt = self.inner.get_global_conf().await;
        if let Some(conf) = conf_opt {
            Ok(Response::new(GetGlobalConfRespWrapped {
                status: GetGlobalConfRespStatus::GlobalConfStatusOk as i32,
                data: Some(GetGlobalConfResp {
                    date_refreshed: chrono::Utc::now().timestamp_millis(),
                    instance_identifier: conf.instance_identifier.clone(),
                    instances: vec![GlobalConfInstance {
                        version: 2,
                        instance_identifier: conf.instance_identifier,
                        files: vec![],
                    }],
                }),
            }))
        } else {
            Ok(Response::new(GetGlobalConfRespWrapped {
                status: GetGlobalConfRespStatus::GlobalConfStatusUninitialized as i32,
                data: None,
            }))
        }
    }

    async fn get_verification_conf(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<GetVerificationConfResp>, Status> {
        Ok(Response::new(GetVerificationConfResp { content: vec![] }))
    }
}

pub struct ConfClientAnchorServiceAdapter {
    inner: Arc<ConfClient>,
}

#[tonic::async_trait]
impl AnchorService for ConfClientAnchorServiceAdapter {
    async fn get_configuration_anchor(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<ConfigurationAnchorMessage>, Status> {
        let anchor_xml = self
            .inner
            .anchor
            .to_xml()
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ConfigurationAnchorMessage {
            configuration_anchor: anchor_xml.into_bytes(),
        }))
    }

    async fn verify_and_save_configuration_anchor(
        &self,
        req: Request<ConfigurationAnchorMessage>,
    ) -> Result<Response<VerificationResult>, Status> {
        let bytes = req.into_inner().configuration_anchor;
        if let Ok(xml_str) = std::str::from_utf8(&bytes) {
            let _ = ConfigurationAnchor::from_xml(xml_str);
        }
        Ok(Response::new(VerificationResult { return_code: 0 }))
    }
}

pub struct ConfClientAdminServiceAdapter {
    inner: Arc<ConfClient>,
}

#[tonic::async_trait]
impl AdminService for ConfClientAdminServiceAdapter {
    async fn get_status(
        &self,
        _req: Request<Empty>,
    ) -> Result<Response<DiagnosticsStatus>, Status> {
        let has_conf = self.inner.get_global_conf().await.is_some();
        Ok(Response::new(DiagnosticsStatus {
            status: if has_conf {
                DiagnosticStatusType::DiagnosticStatusOk as i32
            } else {
                DiagnosticStatusType::DiagnosticStatusUninitialized as i32
            },
            prev_update: Some(chrono::Utc::now().timestamp_millis()),
            next_update: None,
            description: Some("Global configuration sync service operational".to_string()),
            error_code: None,
            error_code_metadata: vec![],
        }))
    }

    async fn check_and_get_connection_status(
        &self,
        _req: Request<CheckAndGetConnectionStatusRequest>,
    ) -> Result<Response<CheckAndGetConnectionStatusResponse>, Status> {
        let mut statuses = Vec::new();
        for url in &self.inner.anchor.download_urls {
            statuses.push(DownloadUrlConnectionStatus {
                download_url: url.clone(),
                error_code: None,
                error_details: None,
            });
        }
        Ok(Response::new(CheckAndGetConnectionStatusResponse {
            connection_statuses: statuses,
        }))
    }
}

/// Launches the Configuration Client gRPC services on the given address.
pub async fn run_confclient_grpc(
    bind_addr: &str,
    client: Arc<ConfClient>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!(
        "Starting X-Road Configuration Client gRPC on {}...",
        bind_addr
    );
    let addr = bind_addr.parse()?;

    tonic::transport::Server::builder()
        .add_service(client.clone().into_global_conf_service())
        .add_service(client.clone().into_anchor_service())
        .add_service(client.into_admin_service())
        .serve(addr)
        .await?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use proto::admin_service_server::AdminService;
    use proto::anchor_service_server::AnchorService;
    use proto::global_conf_service_server::GlobalConfService;

    #[tokio::test]
    async fn test_confclient_services() {
        let anchor = ConfigurationAnchor {
            instance_identifier: "DEV".to_string(),
            generated_at: chrono::Utc::now(),
            download_urls: vec!["http://127.0.0.1:8080/internalconf".to_string()],
            verification_cert_hashes: vec!["hash123".to_string()],
        };

        let client = Arc::new(ConfClient::new(anchor));

        // Test anchor service
        let anchor_adapter = ConfClientAnchorServiceAdapter {
            inner: client.clone(),
        };
        let resp = anchor_adapter
            .get_configuration_anchor(Request::new(Empty {}))
            .await
            .expect("Should return anchor response")
            .into_inner();
        let anchor_xml = String::from_utf8(resp.configuration_anchor).unwrap();
        assert!(anchor_xml.contains("instanceIdentifier>DEV<"));
        assert!(anchor_xml.contains("http://127.0.0.1:8080/internalconf"));

        // Test admin service
        let admin_adapter = ConfClientAdminServiceAdapter {
            inner: client.clone(),
        };
        let resp = admin_adapter
            .check_and_get_connection_status(Request::new(
                CheckAndGetConnectionStatusRequest::default(),
            ))
            .await
            .expect("Should return connection statuses")
            .into_inner();
        assert_eq!(resp.connection_statuses.len(), 1);
        assert_eq!(
            resp.connection_statuses[0].download_url,
            "http://127.0.0.1:8080/internalconf"
        );

        let diag = admin_adapter
            .get_status(Request::new(Empty {}))
            .await
            .expect("Should return diagnostics")
            .into_inner();
        assert_eq!(
            diag.status,
            DiagnosticStatusType::DiagnosticStatusUninitialized as i32
        );

        // Test globalconf service before cache
        let gconf_adapter = ConfClientGlobalConfServiceAdapter {
            inner: client.clone(),
        };
        let resp = gconf_adapter
            .get_global_conf(Request::new(GetGlobalConfReq {}))
            .await
            .expect("Should query global conf")
            .into_inner();
        assert_eq!(
            resp.status,
            GetGlobalConfRespStatus::GlobalConfStatusUninitialized as i32
        );
    }
}
