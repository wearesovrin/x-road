use crate::client_id::ClientId;
use crate::id_trait::XRoadId;
use crate::object_type::XRoadObjectType;
use crate::validator::StrictIdentifierValidator;
use common::{MetaServiceCode, ServiceCode, XRoadError, XRoadResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Identifies an X-Road Service.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ServiceId {
    pub client: ClientId,
    #[serde(rename = "serviceCode", alias = "service_code")]
    pub service_code: ServiceCode,
    #[serde(
        rename = "serviceVersion",
        alias = "service_version",
        skip_serializing_if = "Option::is_none"
    )]
    pub service_version: Option<String>,
}

impl ServiceId {
    /// Creates a validated ServiceId for the given client provider, service code, and optional version.
    pub fn new(
        client: ClientId,
        service_code: impl Into<ServiceCode>,
        service_version: Option<impl Into<String>>,
    ) -> XRoadResult<Self> {
        let code = service_code.into();
        let version = service_version.map(Into::into);

        StrictIdentifierValidator::validate_part("serviceCode", code.as_str())?;
        StrictIdentifierValidator::validate_optional_part("serviceVersion", version.as_deref())?;

        Ok(Self {
            client,
            service_code: code,
            service_version: version,
        })
    }

    /// Creates an unchecked ServiceId without validation.
    pub fn new_unchecked(
        client: ClientId,
        service_code: impl Into<ServiceCode>,
        service_version: Option<impl Into<String>>,
    ) -> Self {
        Self {
            client,
            service_code: service_code.into(),
            service_version: service_version.map(Into::into),
        }
    }

    /// Creates a validated ServiceId from distinct parts.
    pub fn from_parts(
        instance: impl Into<String>,
        class: impl Into<String>,
        code: impl Into<String>,
        subsystem: Option<impl Into<String>>,
        service_code: impl Into<ServiceCode>,
        service_version: Option<impl Into<String>>,
    ) -> XRoadResult<Self> {
        let client = ClientId::new(instance, class, code, subsystem)?;
        Self::new(client, service_code, service_version)
    }

    #[inline]
    pub fn client_id(&self) -> &ClientId {
        &self.client
    }

    #[inline]
    pub fn xroad_instance(&self) -> &str {
        &self.client.xroad_instance
    }

    #[inline]
    pub fn member_class(&self) -> &str {
        &self.client.member_class
    }

    #[inline]
    pub fn member_code(&self) -> &str {
        &self.client.member_code
    }

    #[inline]
    pub fn subsystem_code(&self) -> Option<&str> {
        self.client.subsystem_code.as_deref()
    }

    #[inline]
    pub fn service_code(&self) -> &str {
        self.service_code.as_str()
    }

    #[inline]
    pub fn service_code_enum(&self) -> &ServiceCode {
        &self.service_code
    }

    #[inline]
    pub fn is_meta_service(&self) -> bool {
        self.service_code.is_meta_service()
    }

    #[inline]
    pub fn as_meta_service(&self) -> Option<MetaServiceCode> {
        self.service_code.as_meta_service()
    }

    #[inline]
    pub fn service_version(&self) -> Option<&str> {
        self.service_version.as_deref()
    }
}

impl XRoadId for ServiceId {
    fn object_type(&self) -> XRoadObjectType {
        XRoadObjectType::Service
    }

    fn xroad_instance(&self) -> Option<&str> {
        Some(self.xroad_instance())
    }

    fn fields_for_string_format(&self) -> Vec<&str> {
        let mut fields = vec![self.member_class(), self.member_code()];
        if let Some(sub) = self.subsystem_code() {
            fields.push(sub);
        }
        fields.push(self.service_code());
        if let Some(ver) = self.service_version() {
            fields.push(ver);
        }
        fields
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.client, self.service_code)?;
        if let Some(ver) = &self.service_version {
            write!(f, "/{}", ver)?;
        }
        Ok(())
    }
}

impl FromStr for ServiceId {
    type Err = XRoadError;

    fn from_str(s: &str) -> XRoadResult<Self> {
        let s = s.trim();

        // Strip optional "SERVICE:" prefix
        let raw = if let Some(rest) = s.strip_prefix("SERVICE:") {
            rest
        } else if let Some(rest) = s.strip_prefix("service:") {
            rest
        } else {
            s
        };

        let parts: Vec<&str> = if raw.contains('/') {
            raw.split('/').collect()
        } else {
            raw.split(':').collect()
        };

        match parts.len() {
            // INSTANCE/CLASS/CODE/SERVICE
            4 => {
                let client = ClientId::member(parts[0], parts[1], parts[2])?;
                ServiceId::new(client, parts[3], None::<String>)
            }
            // Either INSTANCE/CLASS/CODE/SUBSYSTEM/SERVICE or INSTANCE/CLASS/CODE/SERVICE/VERSION
            // Upstream standard: 5 parts represents SUBSYSTEM service without version
            5 => {
                let client = ClientId::subsystem(parts[0], parts[1], parts[2], parts[3])?;
                ServiceId::new(client, parts[4], None::<String>)
            }
            // INSTANCE/CLASS/CODE/SUBSYSTEM/SERVICE/VERSION
            6 => {
                let client = ClientId::subsystem(parts[0], parts[1], parts[2], parts[3])?;
                ServiceId::new(client, parts[4], Some(parts[5]))
            }
            _ => Err(XRoadError::IdentifierParse(format!(
                "Invalid ServiceId format '{}': expected 4 to 6 parts",
                s
            ))),
        }
    }
}
