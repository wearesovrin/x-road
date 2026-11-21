use crate::client_id::ClientId;
use crate::object_type::XRoadObjectType;
use crate::server_id::SecurityServerId;
use crate::service_id::ServiceId;
use common::{XRoadError, XRoadResult};
use serde::{Deserialize, Serialize};

pub const XROAD_IDENTIFIERS_NS: &str = "http://x-road.eu/xsd/identifiers";

/// XML representation of an X-Road Client identifier (`id:XRoadClientIdentifierType`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XRoadClientIdentifierType {
    #[serde(rename = "@objectType", alias = "@id:objectType")]
    pub object_type: XRoadObjectType,
    #[serde(rename = "xRoadInstance", alias = "id:xRoadInstance")]
    pub xroad_instance: String,
    #[serde(rename = "memberClass", alias = "id:memberClass")]
    pub member_class: String,
    #[serde(rename = "memberCode", alias = "id:memberCode")]
    pub member_code: String,
    #[serde(rename = "subsystemCode", alias = "id:subsystemCode", skip_serializing_if = "Option::is_none")]
    pub subsystem_code: Option<String>,
}

impl From<&ClientId> for XRoadClientIdentifierType {
    fn from(id: &ClientId) -> Self {
        Self {
            object_type: id.object_type(),
            xroad_instance: id.xroad_instance.clone(),
            member_class: id.member_class.clone(),
            member_code: id.member_code.clone(),
            subsystem_code: id.subsystem_code.clone(),
        }
    }
}

impl TryFrom<XRoadClientIdentifierType> for ClientId {
    type Error = XRoadError;

    fn try_from(xml_id: XRoadClientIdentifierType) -> XRoadResult<Self> {
        match xml_id.object_type {
            XRoadObjectType::Member => {
                if xml_id.subsystem_code.is_some() {
                    return Err(XRoadError::IdentifierParse(
                        "Invalid ClientId: redundant subsystem code present for MEMBER objectType"
                            .to_string(),
                    ));
                }
                ClientId::member(
                    xml_id.xroad_instance,
                    xml_id.member_class,
                    xml_id.member_code,
                )
            }
            XRoadObjectType::Subsystem => {
                let sub = xml_id.subsystem_code.ok_or_else(|| {
                    XRoadError::IdentifierParse(
                        "Invalid ClientId: missing required subsystem code for SUBSYSTEM objectType"
                            .to_string(),
                    )
                })?;
                ClientId::subsystem(
                    xml_id.xroad_instance,
                    xml_id.member_class,
                    xml_id.member_code,
                    sub,
                )
            }
            other => Err(XRoadError::IdentifierParse(format!(
                "Invalid objectType '{}' for client identifier",
                other
            ))),
        }
    }
}

impl XRoadClientIdentifierType {
    /// Formats this client identifier as an XML element with standard `id:` prefix and namespace.
    pub fn to_xml(&self, tag_name: &str) -> String {
        let mut xml = format!(
            r#"<id:{} id:objectType="{}"><id:xRoadInstance>{}</id:xRoadInstance><id:memberClass>{}</id:memberClass><id:memberCode>{}</id:memberCode>"#,
            tag_name, self.object_type, self.xroad_instance, self.member_class, self.member_code
        );
        if let Some(sub) = &self.subsystem_code {
            xml.push_str(&format!("<id:subsystemCode>{}</id:subsystemCode>", sub));
        }
        xml.push_str(&format!("</id:{}>", tag_name));
        xml
    }

    /// Parses an X-Road client identifier from an XML string.
    pub fn parse_xml(xml: &str) -> XRoadResult<ClientId> {
        let parsed: XRoadClientIdentifierType = quick_xml::de::from_str(xml).map_err(|e| {
            XRoadError::IdentifierParse(format!("Failed to parse ClientId XML: {}", e))
        })?;
        ClientId::try_from(parsed)
    }
}

/// XML representation of an X-Road Service identifier (`id:XRoadServiceIdentifierType`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XRoadServiceIdentifierType {
    #[serde(rename = "@objectType", alias = "@id:objectType")]
    pub object_type: XRoadObjectType,
    #[serde(rename = "xRoadInstance", alias = "id:xRoadInstance")]
    pub xroad_instance: String,
    #[serde(rename = "memberClass", alias = "id:memberClass")]
    pub member_class: String,
    #[serde(rename = "memberCode", alias = "id:memberCode")]
    pub member_code: String,
    #[serde(rename = "subsystemCode", alias = "id:subsystemCode", skip_serializing_if = "Option::is_none")]
    pub subsystem_code: Option<String>,
    #[serde(rename = "serviceCode", alias = "id:serviceCode")]
    pub service_code: String,
    #[serde(rename = "serviceVersion", alias = "id:serviceVersion", skip_serializing_if = "Option::is_none")]
    pub service_version: Option<String>,
}

impl From<&ServiceId> for XRoadServiceIdentifierType {
    fn from(s: &ServiceId) -> Self {
        Self {
            object_type: XRoadObjectType::Service,
            xroad_instance: s.xroad_instance().to_string(),
            member_class: s.member_class().to_string(),
            member_code: s.member_code().to_string(),
            subsystem_code: s.subsystem_code().map(ToString::to_string),
            service_code: s.service_code().to_string(),
            service_version: s.service_version().map(ToString::to_string),
        }
    }
}

impl TryFrom<XRoadServiceIdentifierType> for ServiceId {
    type Error = XRoadError;

    fn try_from(xml_id: XRoadServiceIdentifierType) -> XRoadResult<Self> {
        if xml_id.object_type != XRoadObjectType::Service {
            return Err(XRoadError::IdentifierParse(format!(
                "Invalid objectType '{}' for service identifier, expected SERVICE",
                xml_id.object_type
            )));
        }
        let client = ClientId::new(
            xml_id.xroad_instance,
            xml_id.member_class,
            xml_id.member_code,
            xml_id.subsystem_code,
        )?;
        ServiceId::new(client, xml_id.service_code, xml_id.service_version)
    }
}

impl XRoadServiceIdentifierType {
    pub fn to_xml(&self, tag_name: &str) -> String {
        let mut xml = format!(
            r#"<id:{} id:objectType="SERVICE"><id:xRoadInstance>{}</id:xRoadInstance><id:memberClass>{}</id:memberClass><id:memberCode>{}</id:memberCode>"#,
            tag_name, self.xroad_instance, self.member_class, self.member_code
        );
        if let Some(sub) = &self.subsystem_code {
            xml.push_str(&format!("<id:subsystemCode>{}</id:subsystemCode>", sub));
        }
        xml.push_str(&format!(
            "<id:serviceCode>{}</id:serviceCode>",
            self.service_code
        ));
        if let Some(ver) = &self.service_version {
            xml.push_str(&format!("<id:serviceVersion>{}</id:serviceVersion>", ver));
        }
        xml.push_str(&format!("</id:{}>", tag_name));
        xml
    }

    pub fn parse_xml(xml: &str) -> XRoadResult<ServiceId> {
        let parsed: XRoadServiceIdentifierType = quick_xml::de::from_str(xml).map_err(|e| {
            XRoadError::IdentifierParse(format!("Failed to parse ServiceId XML: {}", e))
        })?;
        ServiceId::try_from(parsed)
    }
}

/// XML representation of an X-Road Security Server identifier (`id:XRoadSecurityServerIdentifierType`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XRoadSecurityServerIdentifierType {
    #[serde(rename = "@objectType", alias = "@id:objectType")]
    pub object_type: XRoadObjectType,
    #[serde(rename = "xRoadInstance", alias = "id:xRoadInstance")]
    pub xroad_instance: String,
    #[serde(rename = "memberClass", alias = "id:memberClass")]
    pub member_class: String,
    #[serde(rename = "memberCode", alias = "id:memberCode")]
    pub member_code: String,
    #[serde(rename = "serverCode", alias = "id:serverCode")]
    pub server_code: String,
}

impl From<&SecurityServerId> for XRoadSecurityServerIdentifierType {
    fn from(s: &SecurityServerId) -> Self {
        Self {
            object_type: XRoadObjectType::Server,
            xroad_instance: s.xroad_instance.clone(),
            member_class: s.member_class.clone(),
            member_code: s.member_code.clone(),
            server_code: s.server_code.clone(),
        }
    }
}

impl TryFrom<XRoadSecurityServerIdentifierType> for SecurityServerId {
    type Error = XRoadError;

    fn try_from(xml_id: XRoadSecurityServerIdentifierType) -> XRoadResult<Self> {
        SecurityServerId::new(
            xml_id.xroad_instance,
            xml_id.member_class,
            xml_id.member_code,
            xml_id.server_code,
        )
    }
}
