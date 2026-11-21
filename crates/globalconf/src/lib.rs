use chrono::{DateTime, Utc};
use identifiers::{ClientId, SecurityServerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration anchor pointing to the source of global configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationAnchor {
    pub instance_identifier: String,
    pub generated_at: DateTime<Utc>,
    pub download_urls: Vec<String>,
    pub verification_cert_hashes: Vec<String>,
}

/// Approved Certification Authority in global configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedCa {
    pub name: String,
    pub top_ca_cert: Vec<u8>,
    pub intermediate_ca_certs: Vec<Vec<u8>>,
    pub ocsp_response_freshness: u64,
}

/// Approved Time-Stamping Provider in global configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedTsp {
    pub name: String,
    pub url: String,
    pub cert: Vec<u8>,
}

/// In-memory cache of global configuration data.
#[derive(Debug, Default, Clone)]
pub struct GlobalConf {
    pub instance_identifier: String,
    pub members: HashMap<ClientId, String>,
    pub security_servers: HashMap<SecurityServerId, String>,
    pub approved_cas: Vec<ApprovedCa>,
    pub approved_tsps: Vec<ApprovedTsp>,
}

impl GlobalConf {
    pub fn new(instance_identifier: impl Into<String>) -> Self {
        Self {
            instance_identifier: instance_identifier.into(),
            ..Default::default()
        }
    }

    /// Checks if a client is registered in the global configuration.
    pub fn is_member_registered(&self, client_id: &ClientId) -> bool {
        self.members.contains_key(client_id)
    }

    /// Resolves the address (host:port) for a given Security Server.
    pub fn get_security_server_address(&self, server_id: &SecurityServerId) -> Option<&String> {
        self.security_servers.get(server_id)
    }
}
