use crate::key_purpose::AcmeKeyPurpose;
use serde::{Deserialize, Serialize};

/// Configuration for an X-Road ACME certificate enrollment/renewal provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcmeConfig {
    /// The ACME directory URL (e.g. Let's Encrypt, Smallstep CA, Vault ACME).
    pub directory_url: String,
    /// Contact email address for ACME account registration.
    pub contact_email: String,
    /// Intended usage of certificates issued under this configuration.
    pub key_purpose: AcmeKeyPurpose,
    /// Explicit agreement to the ACME certificate authority's Terms of Service.
    pub terms_agreed: bool,
    /// Optional External Account Binding (EAB) Key Identifier.
    pub eab_kid: Option<String>,
    /// Optional External Account Binding (EAB) HMAC key (Base64 URL-safe encoded).
    pub eab_hmac_key: Option<String>,
}

impl AcmeConfig {
    pub fn lets_encrypt_staging(
        contact_email: impl Into<String>,
        key_purpose: AcmeKeyPurpose,
    ) -> Self {
        Self {
            directory_url: instant_acme::LetsEncrypt::Staging.url().to_string(),
            contact_email: contact_email.into(),
            key_purpose,
            terms_agreed: true,
            eab_kid: None,
            eab_hmac_key: None,
        }
    }

    pub fn lets_encrypt_production(
        contact_email: impl Into<String>,
        key_purpose: AcmeKeyPurpose,
    ) -> Self {
        Self {
            directory_url: instant_acme::LetsEncrypt::Production.url().to_string(),
            contact_email: contact_email.into(),
            key_purpose,
            terms_agreed: true,
            eab_kid: None,
            eab_hmac_key: None,
        }
    }
}
