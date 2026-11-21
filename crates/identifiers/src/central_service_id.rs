use crate::id_trait::XRoadId;
use crate::object_type::XRoadObjectType;
use crate::validator::StrictIdentifierValidator;
use common::{XRoadError, XRoadResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Identifies an X-Road Central Service.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CentralServiceId {
    #[serde(rename = "xRoadInstance", alias = "xroad_instance")]
    pub xroad_instance: String,
    #[serde(rename = "serviceCode", alias = "service_code")]
    pub service_code: String,
}

impl CentralServiceId {
    pub fn new(instance: impl Into<String>, service_code: impl Into<String>) -> XRoadResult<Self> {
        let instance = instance.into();
        let code = service_code.into();
        StrictIdentifierValidator::validate_part("xRoadInstance", &instance)?;
        StrictIdentifierValidator::validate_part("serviceCode", &code)?;
        Ok(Self {
            xroad_instance: instance,
            service_code: code,
        })
    }
}

impl XRoadId for CentralServiceId {
    fn object_type(&self) -> XRoadObjectType {
        XRoadObjectType::CentralService
    }

    fn xroad_instance(&self) -> Option<&str> {
        Some(&self.xroad_instance)
    }

    fn fields_for_string_format(&self) -> Vec<&str> {
        vec![&self.service_code]
    }
}

impl fmt::Display for CentralServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.xroad_instance, self.service_code)
    }
}

impl FromStr for CentralServiceId {
    type Err = XRoadError;

    fn from_str(s: &str) -> XRoadResult<Self> {
        let s = s.trim();
        let raw = if let Some(rest) = s.strip_prefix("CENTRALSERVICE:") {
            rest
        } else if let Some(rest) = s.strip_prefix("centralservice:") {
            rest
        } else {
            s
        };

        let parts: Vec<&str> = if raw.contains('/') {
            raw.split('/').collect()
        } else {
            raw.split(':').collect()
        };

        if parts.len() == 2 {
            CentralServiceId::new(parts[0], parts[1])
        } else {
            Err(XRoadError::IdentifierParse(format!(
                "Invalid CentralServiceId format '{}': expected 2 parts (INSTANCE/SERVICE_CODE)",
                s
            )))
        }
    }
}
