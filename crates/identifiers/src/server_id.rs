use crate::client_id::ClientId;
use crate::id_trait::XRoadId;
use crate::object_type::XRoadObjectType;
use crate::validator::StrictIdentifierValidator;
use common::{XRoadError, XRoadResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Identifies an X-Road Security Server.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SecurityServerId {
    #[serde(rename = "xRoadInstance", alias = "xroad_instance")]
    pub xroad_instance: String,
    #[serde(rename = "memberClass", alias = "member_class")]
    pub member_class: String,
    #[serde(rename = "memberCode", alias = "member_code")]
    pub member_code: String,
    #[serde(rename = "serverCode", alias = "server_code")]
    pub server_code: String,
}

impl SecurityServerId {
    /// Creates a validated SecurityServerId.
    pub fn new(
        instance: impl Into<String>,
        class: impl Into<String>,
        code: impl Into<String>,
        server: impl Into<String>,
    ) -> XRoadResult<Self> {
        let instance = instance.into();
        let class = class.into();
        let code = code.into();
        let server = server.into();

        StrictIdentifierValidator::validate_part("xRoadInstance", &instance)?;
        StrictIdentifierValidator::validate_part("memberClass", &class)?;
        StrictIdentifierValidator::validate_part("memberCode", &code)?;
        StrictIdentifierValidator::validate_part("serverCode", &server)?;

        Ok(Self {
            xroad_instance: instance,
            member_class: class,
            member_code: code,
            server_code: server,
        })
    }

    /// Creates a SecurityServerId from owner Member client and server code.
    pub fn from_owner(owner: &ClientId, server_code: impl Into<String>) -> XRoadResult<Self> {
        Self::new(
            &owner.xroad_instance,
            &owner.member_class,
            &owner.member_code,
            server_code,
        )
    }

    /// Returns the owner Member client identifier of this Security Server.
    pub fn owner(&self) -> ClientId {
        ClientId::member_unchecked(&self.xroad_instance, &self.member_class, &self.member_code)
    }
}

impl XRoadId for SecurityServerId {
    fn object_type(&self) -> XRoadObjectType {
        XRoadObjectType::Server
    }

    fn xroad_instance(&self) -> Option<&str> {
        Some(&self.xroad_instance)
    }

    fn fields_for_string_format(&self) -> Vec<&str> {
        vec![&self.member_class, &self.member_code, &self.server_code]
    }
}

impl fmt::Display for SecurityServerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            self.xroad_instance, self.member_class, self.member_code, self.server_code
        )
    }
}

impl FromStr for SecurityServerId {
    type Err = XRoadError;

    fn from_str(s: &str) -> XRoadResult<Self> {
        let s = s.trim();

        let raw = if let Some(rest) = s.strip_prefix("SERVER:") {
            rest
        } else if let Some(rest) = s.strip_prefix("server:") {
            rest
        } else {
            s
        };

        let parts: Vec<&str> = if raw.contains('/') {
            raw.split('/').collect()
        } else {
            raw.split(':').collect()
        };

        if parts.len() == 4 {
            SecurityServerId::new(parts[0], parts[1], parts[2], parts[3])
        } else {
            Err(XRoadError::IdentifierParse(format!(
                "Invalid SecurityServerId format '{}': expected 4 parts (INSTANCE/CLASS/CODE/SERVER)",
                s
            )))
        }
    }
}
