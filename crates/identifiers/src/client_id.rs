use crate::id_trait::XRoadId;
use crate::object_type::XRoadObjectType;
use crate::validator::StrictIdentifierValidator;
use common::{XRoadError, XRoadResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Identifies an X-Road client (either a Member or a Subsystem within a Member).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientId {
    #[serde(rename = "xRoadInstance", alias = "xroad_instance")]
    pub xroad_instance: String,
    #[serde(rename = "memberClass", alias = "member_class")]
    pub member_class: String,
    #[serde(rename = "memberCode", alias = "member_code")]
    pub member_code: String,
    #[serde(
        rename = "subsystemCode",
        alias = "subsystem_code",
        skip_serializing_if = "Option::is_none"
    )]
    pub subsystem_code: Option<String>,
}

impl ClientId {
    /// Creates a validated Member client identifier (subsystem = None).
    pub fn member(
        instance: impl Into<String>,
        class: impl Into<String>,
        code: impl Into<String>,
    ) -> XRoadResult<Self> {
        let instance = instance.into();
        let class = class.into();
        let code = code.into();

        StrictIdentifierValidator::validate_part("xRoadInstance", &instance)?;
        StrictIdentifierValidator::validate_part("memberClass", &class)?;
        StrictIdentifierValidator::validate_part("memberCode", &code)?;

        Ok(Self {
            xroad_instance: instance,
            member_class: class,
            member_code: code,
            subsystem_code: None,
        })
    }

    /// Creates a validated Subsystem client identifier.
    pub fn subsystem(
        instance: impl Into<String>,
        class: impl Into<String>,
        code: impl Into<String>,
        subsystem: impl Into<String>,
    ) -> XRoadResult<Self> {
        let instance = instance.into();
        let class = class.into();
        let code = code.into();
        let subsystem = subsystem.into();

        StrictIdentifierValidator::validate_part("xRoadInstance", &instance)?;
        StrictIdentifierValidator::validate_part("memberClass", &class)?;
        StrictIdentifierValidator::validate_part("memberCode", &code)?;
        StrictIdentifierValidator::validate_part("subsystemCode", &subsystem)?;

        Ok(Self {
            xroad_instance: instance,
            member_class: class,
            member_code: code,
            subsystem_code: Some(subsystem),
        })
    }

    /// Creates a validated ClientId (either member or subsystem depending on optional subsystem argument).
    pub fn new(
        instance: impl Into<String>,
        class: impl Into<String>,
        code: impl Into<String>,
        subsystem: Option<impl Into<String>>,
    ) -> XRoadResult<Self> {
        match subsystem {
            Some(sub) => Self::subsystem(instance, class, code, sub),
            None => Self::member(instance, class, code),
        }
    }

    /// Creates an unchecked Member client identifier without validation (for tests/trusted contexts).
    pub fn member_unchecked(
        instance: impl Into<String>,
        class: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            xroad_instance: instance.into(),
            member_class: class.into(),
            member_code: code.into(),
            subsystem_code: None,
        }
    }

    /// Creates an unchecked Subsystem client identifier without validation.
    pub fn subsystem_unchecked(
        instance: impl Into<String>,
        class: impl Into<String>,
        code: impl Into<String>,
        subsystem: impl Into<String>,
    ) -> Self {
        Self {
            xroad_instance: instance.into(),
            member_class: class.into(),
            member_code: code.into(),
            subsystem_code: Some(subsystem.into()),
        }
    }

    /// Returns the object type (`Member` if `subsystem_code` is None, `Subsystem` otherwise).
    #[inline]
    pub fn object_type(&self) -> XRoadObjectType {
        if self.subsystem_code.is_some() {
            XRoadObjectType::Subsystem
        } else {
            XRoadObjectType::Member
        }
    }

    /// True if this client represents an X-Road Member.
    #[inline]
    pub fn is_member(&self) -> bool {
        self.subsystem_code.is_none()
    }

    /// True if this client represents an X-Road Subsystem.
    #[inline]
    pub fn is_subsystem(&self) -> bool {
        self.subsystem_code.is_some()
    }

    /// Returns the base Member client identifier (with `subsystem_code = None`).
    pub fn member_id(&self) -> ClientId {
        ClientId {
            xroad_instance: self.xroad_instance.clone(),
            member_class: self.member_class.clone(),
            member_code: self.member_code.clone(),
            subsystem_code: None,
        }
    }

    /// Determines whether the given member is a subsystem of this client.
    ///
    /// Upstream specification: Returns true iff `self` is a Subsystem and `member` is a
    /// Member and both share instance, member_class, and member_code.
    pub fn subsystem_contains_member(&self, member: &ClientId) -> bool {
        self.is_subsystem() && member.is_member() && self.member_equals(member)
    }

    /// Determines if another client refers to the same X-Road member (ignoring subsystem).
    pub fn member_equals(&self, other: &ClientId) -> bool {
        self.xroad_instance == other.xroad_instance
            && self.member_class == other.member_class
            && self.member_code == other.member_code
    }
}

impl XRoadId for ClientId {
    fn object_type(&self) -> XRoadObjectType {
        self.object_type()
    }

    fn xroad_instance(&self) -> Option<&str> {
        Some(&self.xroad_instance)
    }

    fn fields_for_string_format(&self) -> Vec<&str> {
        let mut fields = vec![self.member_class.as_str(), self.member_code.as_str()];
        if let Some(sub) = &self.subsystem_code {
            fields.push(sub.as_str());
        }
        fields
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.xroad_instance, self.member_class, self.member_code
        )?;
        if let Some(sub) = &self.subsystem_code {
            write!(f, "/{}", sub)?;
        }
        Ok(())
    }
}

impl FromStr for ClientId {
    type Err = XRoadError;

    fn from_str(s: &str) -> XRoadResult<Self> {
        let s = s.trim();

        // Strip optional prefix (e.g. "MEMBER:" or "SUBSYSTEM:")
        let (raw_type, raw_id) = if let Some((prefix, rest)) = s.split_once(':') {
            if prefix.eq_ignore_ascii_case("MEMBER") || prefix.eq_ignore_ascii_case("SUBSYSTEM") {
                (Some(prefix), rest)
            } else {
                (None, s)
            }
        } else {
            (None, s)
        };

        // Split either by '/' or ':'
        let parts: Vec<&str> = if raw_id.contains('/') {
            raw_id.split('/').collect()
        } else {
            raw_id.split(':').collect()
        };

        match parts.len() {
            3 => {
                if let Some(p) = raw_type
                    && p.eq_ignore_ascii_case("SUBSYSTEM")
                {
                    return Err(XRoadError::IdentifierParse(format!(
                        "Invalid Subsystem identifier '{}': missing subsystem code",
                        s
                    )));
                }
                ClientId::member(parts[0], parts[1], parts[2])
            }
            4 => {
                if let Some(p) = raw_type
                    && p.eq_ignore_ascii_case("MEMBER")
                {
                    return Err(XRoadError::IdentifierParse(format!(
                        "Invalid Member identifier '{}': redundant subsystem code element",
                        s
                    )));
                }
                ClientId::subsystem(parts[0], parts[1], parts[2], parts[3])
            }
            _ => Err(XRoadError::IdentifierParse(format!(
                "Invalid ClientId string format '{}': expected 3 (member) or 4 (subsystem) parts",
                s
            ))),
        }
    }
}
