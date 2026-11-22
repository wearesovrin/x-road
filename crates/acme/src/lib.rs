pub mod challenge;
pub mod client;
pub mod config;
pub mod key_purpose;
pub mod renewal;

pub use challenge::AcmeChallengeStore;
pub use client::XRoadAcmeClient;
pub use config::AcmeConfig;
pub use key_purpose::AcmeKeyPurpose;
pub use renewal::{AcmeRenewalWorker, TrackedCertificate};

// Re-export core instant-acme types for convenience
pub use instant_acme::{Account, AccountCredentials, LetsEncrypt, OrderStatus};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acme_key_purpose_strum_and_serde() {
        use std::str::FromStr;

        // String formatting and strum conversion
        assert_eq!(AcmeKeyPurpose::Authentication.to_string(), "AUTHENTICATION");
        assert_eq!(AcmeKeyPurpose::Signing.to_string(), "SIGNING");
        assert_eq!(
            AcmeKeyPurpose::from_str("authentication").unwrap(),
            AcmeKeyPurpose::Authentication
        );
        assert_eq!(
            AcmeKeyPurpose::from_str("SIGNING").unwrap(),
            AcmeKeyPurpose::Signing
        );

        // Serde JSON
        let json = serde_json::to_string(&AcmeKeyPurpose::Authentication).unwrap();
        assert_eq!(json, r#""AUTHENTICATION""#);
        let deserialized: AcmeKeyPurpose = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AcmeKeyPurpose::Authentication);
    }

    #[test]
    fn test_acme_challenge_store() {
        let store = AcmeChallengeStore::new();

        assert_eq!(store.get("token123"), None);

        store.insert("token123", "token123.auth-thumbprint-xyz");
        assert_eq!(
            store.get("token123").as_deref(),
            Some("token123.auth-thumbprint-xyz")
        );

        assert_eq!(
            store.remove("token123").as_deref(),
            Some("token123.auth-thumbprint-xyz")
        );
        assert_eq!(store.get("token123"), None);
    }

    #[test]
    fn test_acme_config_builders() {
        let staging =
            AcmeConfig::lets_encrypt_staging("admin@example.org", AcmeKeyPurpose::Authentication);
        assert!(staging.directory_url.contains("staging"));
        assert_eq!(staging.contact_email, "admin@example.org");
        assert_eq!(staging.key_purpose, AcmeKeyPurpose::Authentication);
        assert!(staging.terms_agreed);

        let prod = AcmeConfig::lets_encrypt_production("secops@sovrin.ee", AcmeKeyPurpose::Signing);
        assert!(!prod.directory_url.contains("staging"));
        assert_eq!(prod.contact_email, "secops@sovrin.ee");
        assert_eq!(prod.key_purpose, AcmeKeyPurpose::Signing);
    }

    #[tokio::test]
    async fn test_acme_renewal_worker_tracking() {
        use chrono::Utc;
        use std::sync::Arc;
        use std::time::Duration;

        let config =
            AcmeConfig::lets_encrypt_staging("admin@test.org", AcmeKeyPurpose::Authentication);
        let client = Arc::new(XRoadAcmeClient::new(config, AcmeChallengeStore::new()));
        let worker = AcmeRenewalWorker::new(client, None);

        let expiring_cert = TrackedCertificate {
            id: "cert-1".to_string(),
            common_name: "ss1.example.com".to_string(),
            sans: vec!["ss1.example.com".to_string()],
            csr_der: vec![1, 2, 3],
            not_after: Utc::now() + chrono::Duration::days(5),
            renewed_cert_pem: None,
        };

        let valid_cert = TrackedCertificate {
            id: "cert-2".to_string(),
            common_name: "ss2.example.com".to_string(),
            sans: vec!["ss2.example.com".to_string()],
            csr_der: vec![4, 5, 6],
            not_after: Utc::now() + chrono::Duration::days(60),
            renewed_cert_pem: None,
        };

        assert!(expiring_cert.is_expiring_soon(Duration::from_secs(30 * 86400)));
        assert!(!valid_cert.is_expiring_soon(Duration::from_secs(30 * 86400)));

        worker.track_certificate(expiring_cert).await;
        worker.track_certificate(valid_cert).await;

        let tracked = worker.get_tracked_certificates().await;
        assert_eq!(tracked.len(), 2);
    }
}
