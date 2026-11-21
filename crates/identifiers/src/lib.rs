use common::{XRoadError, XRoadResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// X-Road Object Type indicating the hierarchy of an identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum XRoadObjectType {
    Server,
    Service,
    Member,
    Subsystem,
    GlobalGroup,
    LocalGroup,
    CentralService,
}

/// Identifies an X-Road client (either a Member or a Subsystem within a Member).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId {
    pub xroad_instance: String,
    pub member_class: String,
    pub member_code: String,
    pub subsystem_code: Option<String>,
}

impl ClientId {
    pub fn member(
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

    pub fn subsystem(
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

    pub fn object_type(&self) -> XRoadObjectType {
        if self.subsystem_code.is_some() {
            XRoadObjectType::Subsystem
        } else {
            XRoadObjectType::Member
        }
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
        let parts: Vec<&str> = s.split('/').collect();
        match parts.len() {
            3 => Ok(ClientId::member(parts[0], parts[1], parts[2])),
            4 => Ok(ClientId::subsystem(parts[0], parts[1], parts[2], parts[3])),
            _ => Err(XRoadError::IdentifierParse(format!(
                "Invalid ClientId string format '{}', expected 3 or 4 slash-separated parts",
                s
            ))),
        }
    }
}

/// Identifies an X-Road service.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceId {
    pub client: ClientId,
    pub service_code: String,
    pub service_version: Option<String>,
}

impl ServiceId {
    pub fn new(
        client: ClientId,
        service_code: impl Into<String>,
        service_version: Option<impl Into<String>>,
    ) -> Self {
        Self {
            client,
            service_code: service_code.into(),
            service_version: service_version.map(Into::into),
        }
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
        let parts: Vec<&str> = s.split('/').collect();
        match parts.len() {
            4 => {
                let client = ClientId::member(parts[0], parts[1], parts[2]);
                Ok(ServiceId {
                    client,
                    service_code: parts[3].to_string(),
                    service_version: None,
                })
            }
            5 => {
                let client = ClientId::subsystem(parts[0], parts[1], parts[2], parts[3]);
                Ok(ServiceId {
                    client,
                    service_code: parts[4].to_string(),
                    service_version: None,
                })
            }
            6 => {
                let client = ClientId::subsystem(parts[0], parts[1], parts[2], parts[3]);
                Ok(ServiceId {
                    client,
                    service_code: parts[4].to_string(),
                    service_version: Some(parts[5].to_string()),
                })
            }
            _ => Err(XRoadError::IdentifierParse(format!(
                "Invalid ServiceId format '{}', expected 4 to 6 slash-separated parts",
                s
            ))),
        }
    }
}

/// Identifies an X-Road Security Server.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecurityServerId {
    pub xroad_instance: String,
    pub member_class: String,
    pub member_code: String,
    pub server_code: String,
}

impl SecurityServerId {
    pub fn new(
        instance: impl Into<String>,
        class: impl Into<String>,
        code: impl Into<String>,
        server: impl Into<String>,
    ) -> Self {
        Self {
            xroad_instance: instance.into(),
            member_class: class.into(),
            member_code: code.into(),
            server_code: server.into(),
        }
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
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() == 4 {
            Ok(SecurityServerId::new(
                parts[0], parts[1], parts[2], parts[3],
            ))
        } else {
            Err(XRoadError::IdentifierParse(format!(
                "Invalid SecurityServerId format '{}', expected 4 slash-separated parts",
                s
            )))
        }
    }
}

/// Identifies an X-Road Global Group.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GlobalGroupId {
    pub xroad_instance: String,
    pub group_code: String,
}

impl fmt::Display for GlobalGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.xroad_instance, self.group_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_id_parsing_and_display() {
        let member_str = "DEV/COM/12345678";
        let member_id: ClientId = member_str.parse().unwrap();
        assert_eq!(member_id.xroad_instance, "DEV");
        assert_eq!(member_id.member_class, "COM");
        assert_eq!(member_id.member_code, "12345678");
        assert_eq!(member_id.subsystem_code, None);
        assert_eq!(member_id.object_type(), XRoadObjectType::Member);
        assert_eq!(member_id.to_string(), member_str);

        let sub_str = "DEV/COM/12345678/billing";
        let sub_id: ClientId = sub_str.parse().unwrap();
        assert_eq!(sub_id.subsystem_code.as_deref(), Some("billing"));
        assert_eq!(sub_id.object_type(), XRoadObjectType::Subsystem);
        assert_eq!(sub_id.to_string(), sub_str);
    }

    #[test]
    fn test_service_id_parsing_and_display() {
        let svc_str = "DEV/COM/12345678/billing/getInvoices/v1";
        let svc_id: ServiceId = svc_str.parse().unwrap();
        assert_eq!(svc_id.client.subsystem_code.as_deref(), Some("billing"));
        assert_eq!(svc_id.service_code, "getInvoices");
        assert_eq!(svc_id.service_version.as_deref(), Some("v1"));
        assert_eq!(svc_id.to_string(), svc_str);
    }

    #[test]
    fn test_security_server_id() {
        let server_str = "DEV/COM/12345678/ss1";
        let server_id: SecurityServerId = server_str.parse().unwrap();
        assert_eq!(server_id.server_code, "ss1");
        assert_eq!(server_id.to_string(), server_str);
    }
}
