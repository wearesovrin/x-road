pub mod challenge;
pub mod client;
pub mod config;
pub mod key_purpose;

pub use challenge::AcmeChallengeStore;
pub use client::XRoadAcmeClient;
pub use config::AcmeConfig;
pub use key_purpose::AcmeKeyPurpose;

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
        let staging = AcmeConfig::lets_encrypt_staging("admin@example.org", AcmeKeyPurpose::Authentication);
        assert!(staging.directory_url.contains("staging"));
        assert_eq!(staging.contact_email, "admin@example.org");
        assert_eq!(staging.key_purpose, AcmeKeyPurpose::Authentication);
        assert!(staging.terms_agreed);

        let prod = AcmeConfig::lets_encrypt_production("secops@sovrin.ee", AcmeKeyPurpose::Signing);
        assert!(!prod.directory_url.contains("staging"));
        assert_eq!(prod.contact_email, "secops@sovrin.ee");
        assert_eq!(prod.key_purpose, AcmeKeyPurpose::Signing);
    }
}
