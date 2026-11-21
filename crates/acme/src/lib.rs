use serde::{Deserialize, Serialize};

/// ACME account and client configuration for automated certificate management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcmeConfig {
    pub directory_url: String,
    pub contact_email: String,
    pub terms_agreed: bool,
}

/// Represents an active ACME order for security server authentication or signing certificates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcmeOrder {
    pub order_url: String,
    pub status: String,
    pub finalize_url: String,
    pub certificate_url: Option<String>,
}
