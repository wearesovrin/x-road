use crate::id_trait::XRoadId;
use crate::object_type::XRoadObjectType;
use crate::validator::StrictIdentifierValidator;
use common::{XRoadError, XRoadResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Identifies an X-Road Global Group across an instance.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GlobalGroupId {
    #[serde(rename = "xRoadInstance", alias = "xroad_instance")]
    pub xroad_instance: String,
    #[serde(rename = "groupCode", alias = "group_code")]
    pub group_code: String,
}

impl GlobalGroupId {
    pub fn new(instance: impl Into<String>, group_code: impl Into<String>) -> XRoadResult<Self> {
        let instance = instance.into();
        let code = group_code.into();
        StrictIdentifierValidator::validate_part("xRoadInstance", &instance)?;
        StrictIdentifierValidator::validate_part("groupCode", &code)?;
        Ok(Self {
            xroad_instance: instance,
            group_code: code,
        })
    }
}

impl XRoadId for GlobalGroupId {
    fn object_type(&self) -> XRoadObjectType {
        XRoadObjectType::GlobalGroup
    }

    fn xroad_instance(&self) -> Option<&str> {
        Some(&self.xroad_instance)
    }

    fn fields_for_string_format(&self) -> Vec<&str> {
        vec![&self.group_code]
    }
}

impl fmt::Display for GlobalGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.xroad_instance, self.group_code)
    }
}

impl FromStr for GlobalGroupId {
    type Err = XRoadError;

    fn from_str(s: &str) -> XRoadResult<Self> {
        let s = s.trim();
        let raw = if let Some(rest) = s.strip_prefix("GLOBALGROUP:") {
            rest
        } else if let Some(rest) = s.strip_prefix("globalgroup:") {
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
            GlobalGroupId::new(parts[0], parts[1])
        } else {
            Err(XRoadError::IdentifierParse(format!(
                "Invalid GlobalGroupId format '{}': expected 2 parts (INSTANCE/GROUP_CODE)",
                s
            )))
        }
    }
}

/// Identifies a Local Group defined within a Security Server.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LocalGroupId {
    #[serde(rename = "groupCode", alias = "group_code")]
    pub group_code: String,
}

impl LocalGroupId {
    pub fn new(group_code: impl Into<String>) -> XRoadResult<Self> {
        let code = group_code.into();
        StrictIdentifierValidator::validate_part("groupCode", &code)?;
        Ok(Self { group_code: code })
    }
}

impl XRoadId for LocalGroupId {
    fn object_type(&self) -> XRoadObjectType {
        XRoadObjectType::LocalGroup
    }

    fn xroad_instance(&self) -> Option<&str> {
        None
    }

    fn fields_for_string_format(&self) -> Vec<&str> {
        vec![&self.group_code]
    }
}

impl fmt::Display for LocalGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.group_code)
    }
}

impl FromStr for LocalGroupId {
    type Err = XRoadError;

    fn from_str(s: &str) -> XRoadResult<Self> {
        let s = s.trim();
        let raw = if let Some(rest) = s.strip_prefix("LOCALGROUP:") {
            rest
        } else if let Some(rest) = s.strip_prefix("localgroup:") {
            rest
        } else {
            s
        };

        LocalGroupId::new(raw)
    }
}
