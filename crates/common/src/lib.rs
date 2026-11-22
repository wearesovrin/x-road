pub mod enums;
pub mod error;
pub mod time;

pub use enums::*;
pub use error::{FaultCode, XRoadError, XRoadResult};
pub use time::CallBudget;

/// Standard X-Road user roles.
pub mod roles {
    pub const XROAD_SECURITY_OFFICER: &str = "XROAD_SECURITY_OFFICER";
    pub const XROAD_REGISTRATION_OFFICER: &str = "XROAD_REGISTRATION_OFFICER";
    pub const XROAD_SERVICE_ADMINISTRATOR: &str = "XROAD_SERVICE_ADMINISTRATOR";
    pub const XROAD_SYSTEM_ADMINISTRATOR: &str = "XROAD_SYSTEM_ADMINISTRATOR";
    pub const XROAD_SECURITYSERVER_OBSERVER: &str = "XROAD_SECURITYSERVER_OBSERVER";
    pub const XROAD_MANAGEMENT_SERVICE: &str = "XROAD_MANAGEMENT_SERVICE";
    pub const XROAD_CENTRAL_SERVICE_ADMINISTRATOR: &str = "XROAD_CENTRAL_SERVICE_ADMINISTRATOR";

    pub const ALL_SECURITY_SERVER_ROLES: &[&str] = &[
        XROAD_SECURITY_OFFICER,
        XROAD_REGISTRATION_OFFICER,
        XROAD_SERVICE_ADMINISTRATOR,
        XROAD_SYSTEM_ADMINISTRATOR,
        XROAD_SECURITYSERVER_OBSERVER,
        XROAD_MANAGEMENT_SERVICE,
    ];

    pub const ALL_CENTRAL_SERVER_ROLES: &[&str] = &[
        XROAD_SECURITY_OFFICER,
        XROAD_REGISTRATION_OFFICER,
        XROAD_SYSTEM_ADMINISTRATOR,
        XROAD_CENTRAL_SERVICE_ADMINISTRATOR,
        XROAD_MANAGEMENT_SERVICE,
    ];
}

/// Authentication and principal representation for admin APIs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AuthenticatedUser {
    pub key_id: Option<String>,
    pub roles: Vec<String>,
}

impl AuthenticatedUser {
    pub fn new(key_id: Option<String>, roles: Vec<String>) -> Self {
        Self { key_id, roles }
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r.eq_ignore_ascii_case(role))
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|r| self.has_role(r))
    }
}

/// Default X-Road protocol version.
pub const XROAD_PROTOCOL_VERSION: &str = "4.0";

/// Standard X-Road HTTP headers.
pub mod headers {
    pub const X_ROAD_CLIENT: &str = "x-road-client";
    pub const X_ROAD_SERVICE: &str = "x-road-service";
    pub const X_ROAD_ID: &str = "x-road-id";
    pub const X_ROAD_USER_ID: &str = "x-road-userid";
    pub const X_ROAD_ISSUE: &str = "x-road-issue";
    pub const X_ROAD_PROTOCOL_VERSION: &str = "x-road-protocol-version";
    pub const X_ROAD_REPRESENTED_PARTY: &str = "x-road-represented-party";
    pub const X_ROAD_REQUEST_ID: &str = "x-road-request-id";
    pub const X_ROAD_ORIGINAL_CONTENT_TYPE: &str = "x-original-content-type";
    pub const X_HASH_ALGORITHM: &str = "x-hash-algorithm";

    /// Parses an API key token from the HTTP Authorization header (e.g. `X-Road-ApiKey token=<key>`).
    pub fn parse_api_key_header(auth_header: &str) -> Option<&str> {
        let trimmed = auth_header.trim();
        for prefix in &["X-Road-ApiKey token=", "x-road-apikey token=", "Bearer "] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                return Some(rest.trim());
            }
        }
        if trimmed
            .to_ascii_lowercase()
            .starts_with("x-road-apikey token=")
        {
            return Some(trimmed[20..].trim());
        }
        None
    }
}
