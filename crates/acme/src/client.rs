use crate::challenge::AcmeChallengeStore;
use crate::config::AcmeConfig;
use common::{XRoadError, XRoadResult};
use instant_acme::{
    Account, AccountCredentials, ChallengeType, ExternalAccountKey, Identifier, NewAccount,
    NewOrder, OrderStatus,
};
use std::time::Duration;
use tracing::{debug, info, warn};

/// High-level ACME client for X-Road certificate enrollment and renewal.
pub struct XRoadAcmeClient {
    pub config: AcmeConfig,
    pub challenge_store: AcmeChallengeStore,
}

impl XRoadAcmeClient {
    pub fn new(config: AcmeConfig, challenge_store: AcmeChallengeStore) -> Self {
        Self {
            config,
            challenge_store,
        }
    }

    /// Creates a new ACME account with the configured directory and credentials.
    pub async fn create_account(&self) -> XRoadResult<(Account, AccountCredentials)> {
        let contact_email = format!("mailto:{}", self.config.contact_email);
        let contact = [contact_email.as_str()];

        let new_account = NewAccount {
            contact: &contact,
            terms_of_service_agreed: self.config.terms_agreed,
            only_return_existing: false,
        };

        let eab = match (&self.config.eab_kid, &self.config.eab_hmac_key) {
            (Some(kid), Some(hmac)) => {
                use base64::Engine;
                let hmac_bytes = base64::prelude::BASE64_URL_SAFE_NO_PAD
                    .decode(hmac)
                    .or_else(|_| base64::prelude::BASE64_STANDARD.decode(hmac))
                    .map_err(|e| {
                        XRoadError::Configuration(format!("Invalid EAB HMAC key encoding: {}", e))
                    })?;
                Some(ExternalAccountKey::new(kid.clone(), &hmac_bytes))
            }
            _ => None,
        };

        let builder = Account::builder()
            .map_err(|e| XRoadError::Certificate(format!("Failed to build ACME account: {}", e)))?;

        builder
            .create(&new_account, self.config.directory_url.clone(), eab.as_ref())
            .await
            .map_err(|e| XRoadError::Certificate(format!("ACME account creation failed: {}", e)))
    }

    /// Restores an existing account from serialized credentials.
    pub async fn load_account(&self, credentials: AccountCredentials) -> XRoadResult<Account> {
        let builder = Account::builder()
            .map_err(|e| XRoadError::Certificate(format!("Failed to build ACME account: {}", e)))?;

        builder
            .from_credentials(credentials)
            .await
            .map_err(|e| XRoadError::Certificate(format!("Failed to load account from credentials: {}", e)))
    }

    /// Places an ACME order for the given domain and SANs, fulfills HTTP-01 challenges,
    /// submits the provided Certificate Signing Request (CSR) in DER format, and returns the certificate chain PEM.
    pub async fn order_certificate(
        &self,
        account: &Account,
        common_name: &str,
        sans: &[&str],
        csr_der: &[u8],
    ) -> XRoadResult<String> {
        let mut identifiers = vec![Identifier::Dns(common_name.to_string())];
        for san in sans {
            if *san != common_name {
                identifiers.push(Identifier::Dns(san.to_string()));
            }
        }

        info!(
            common_name = %common_name,
            sans_count = sans.len(),
            purpose = %self.config.key_purpose,
            "Placing ACME order for certificate"
        );

        let mut order = account
            .new_order(&NewOrder::new(&identifiers))
            .await
            .map_err(|e| XRoadError::Certificate(format!("Failed to create ACME order: {}", e)))?;

        // Fulfill challenges
        let mut authorizations = order.authorizations();
        while let Some(auth_result) = authorizations.next().await {
            let mut auth = auth_result
                .map_err(|e| XRoadError::Certificate(format!("Authorization fetch error: {}", e)))?;

            if let Some(mut challenge) = auth.challenge(ChallengeType::Http01) {
                let key_auth = challenge.key_authorization();
                let token = challenge.token.clone();

                debug!(token = %token, "Registering HTTP-01 challenge in store");
                self.challenge_store.insert(&token, key_auth.as_str());

                // Notify ACME server that the challenge is ready
                challenge
                    .set_ready()
                    .await
                    .map_err(|e| XRoadError::Certificate(format!("Failed to set challenge ready: {}", e)))?;
            } else {
                warn!("No HTTP-01 challenge found for authorization");
            }
        }

        // Wait for order to become Ready for CSR finalization
        let mut attempts = 0;
        let max_attempts = 40;
        loop {
            let state = order
                .refresh()
                .await
                .map_err(|e| XRoadError::Certificate(format!("Failed to refresh order: {}", e)))?;

            match state.status {
                OrderStatus::Ready | OrderStatus::Valid => break,
                OrderStatus::Invalid => {
                    return Err(XRoadError::Certificate(
                        "ACME order became Invalid during challenge validation".to_string(),
                    ));
                }
                _ => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(XRoadError::Certificate(
                            "Timed out waiting for ACME order readiness".to_string(),
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }

        // Finalize order with raw CSR DER bytes from Security Server / Signer
        info!("Finalizing ACME order with raw CSR");
        order
            .finalize_csr(csr_der)
            .await
            .map_err(|e| XRoadError::Certificate(format!("Failed to finalize CSR on ACME order: {}", e)))?;

        // Download certificate chain
        let mut cert_attempts = 0;
        loop {
            if let Some(cert_pem) = order
                .certificate()
                .await
                .map_err(|e| XRoadError::Certificate(format!("Failed to download certificate: {}", e)))?
            {
                info!("Successfully obtained certificate chain from ACME provider");
                return Ok(cert_pem);
            }

            cert_attempts += 1;
            if cert_attempts >= 40 {
                return Err(XRoadError::Certificate(
                    "Timed out waiting for ACME certificate issuance".to_string(),
                ));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}
