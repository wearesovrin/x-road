use crate::client::XRoadAcmeClient;
use chrono::{DateTime, Utc};
use common::{XRoadError, XRoadResult};
use instant_acme::Account;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// A certificate tracked for automated renewal.
#[derive(Debug, Clone)]
pub struct TrackedCertificate {
    pub id: String,
    pub common_name: String,
    pub sans: Vec<String>,
    pub csr_der: Vec<u8>,
    pub not_after: DateTime<Utc>,
    pub renewed_cert_pem: Option<String>,
}

impl TrackedCertificate {
    /// Determines whether the certificate is within the renewal window (default: 30 days before expiry).
    pub fn is_expiring_soon(&self, renew_before: Duration) -> bool {
        let now = Utc::now();
        let expiry = self.not_after;
        if expiry <= now {
            return true;
        }
        let remaining = (expiry - now).to_std().unwrap_or(Duration::ZERO);
        remaining <= renew_before
    }
}

/// Automated background worker that monitors certificate expiration and issues ACME renewals.
pub struct AcmeRenewalWorker {
    client: Arc<XRoadAcmeClient>,
    account: Arc<RwLock<Option<Account>>>,
    tracked_certs: Arc<RwLock<Vec<TrackedCertificate>>>,
    renew_before: Duration,
    check_interval: Duration,
}

impl AcmeRenewalWorker {
    pub fn new(client: Arc<XRoadAcmeClient>, account: Option<Account>) -> Self {
        Self {
            client,
            account: Arc::new(RwLock::new(account)),
            tracked_certs: Arc::new(RwLock::new(Vec::new())),
            renew_before: Duration::from_secs(30 * 86400), // 30 days
            check_interval: Duration::from_secs(3600),     // 1 hour
        }
    }

    pub fn with_renew_before(mut self, duration: Duration) -> Self {
        self.renew_before = duration;
        self
    }

    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    pub async fn track_certificate(&self, cert: TrackedCertificate) {
        let mut certs = self.tracked_certs.write().await;
        certs.retain(|c| c.id != cert.id);
        certs.push(cert);
    }

    pub async fn get_tracked_certificates(&self) -> Vec<TrackedCertificate> {
        self.tracked_certs.read().await.clone()
    }

    /// Performs a single renewal scan over all tracked certificates.
    pub async fn check_and_renew_all(&self) -> Vec<(String, XRoadResult<String>)> {
        let certs = self.tracked_certs.read().await.clone();
        let mut results = Vec::new();

        for cert in certs {
            if cert.is_expiring_soon(self.renew_before) {
                info!(
                    cert_id = %cert.id,
                    common_name = %cert.common_name,
                    not_after = %cert.not_after,
                    "Certificate is expiring soon, triggering automated renewal"
                );

                let account_guard = self.account.read().await;
                if let Some(ref account) = *account_guard {
                    let sans_strs: Vec<&str> = cert.sans.iter().map(String::as_str).collect();
                    match self
                        .client
                        .order_certificate(account, &cert.common_name, &sans_strs, &cert.csr_der)
                        .await
                    {
                        Ok(new_pem) => {
                            info!(cert_id = %cert.id, "Successfully renewed certificate via ACME");
                            let mut write_guard = self.tracked_certs.write().await;
                            if let Some(found) = write_guard.iter_mut().find(|c| c.id == cert.id) {
                                found.renewed_cert_pem = Some(new_pem.clone());
                            }
                            results.push((cert.id, Ok(new_pem)));
                        }
                        Err(e) => {
                            error!(cert_id = %cert.id, error = %e, "Failed to renew certificate via ACME");
                            results.push((cert.id, Err(e)));
                        }
                    }
                } else {
                    warn!("Cannot renew certificate: no ACME account configured in worker");
                    results.push((
                        cert.id,
                        Err(XRoadError::Certificate(
                            "No ACME account configured".to_string(),
                        )),
                    ));
                }
            }
        }

        results
    }

    /// Spawns the background renewal loop task.
    pub fn spawn(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Starting ACME background renewal worker loop");
            loop {
                let _ = self.check_and_renew_all().await;
                tokio::time::sleep(self.check_interval).await;
            }
        })
    }
}
