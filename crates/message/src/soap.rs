use crate::XRoadHeaders;
use common::{XROAD_PROTOCOL_VERSION, XRoadError, XRoadResult};
use identifiers::{ClientId, ServiceId, XRoadClientIdentifierType, XRoadServiceIdentifierType};
use serde::{Deserialize, Serialize};

pub const SOAP_ENV_NS: &str = "http://schemas.xmlsoap.org/soap/envelope/";
pub const XROAD_NS: &str = "http://x-road.eu/xsd/xroad.xsd";
pub const XROAD_ID_NS: &str = "http://x-road.eu/xsd/identifiers";

/// X-Road SOAP Header structure, mapping to `<SOAP-ENV:Header>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoapHeaderXml {
    #[serde(rename = "xroad:client", alias = "client")]
    pub client: XRoadClientIdentifierType,
    #[serde(rename = "xroad:service", alias = "service")]
    pub service: XRoadServiceIdentifierType,
    #[serde(rename = "xroad:id", alias = "id")]
    pub id: String,
    #[serde(
        rename = "xroad:userId",
        alias = "userId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub user_id: Option<String>,
    #[serde(
        rename = "xroad:issue",
        alias = "issue",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub issue: Option<String>,
    #[serde(rename = "xroad:protocolVersion", alias = "protocolVersion")]
    pub protocol_version: String,
    #[serde(
        rename = "xroad:representedParty",
        alias = "representedParty",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub represented_party: Option<String>,
}

impl SoapHeaderXml {
    pub fn from_xroad_headers(headers: &XRoadHeaders) -> Self {
        Self {
            client: XRoadClientIdentifierType::from(&headers.client),
            service: XRoadServiceIdentifierType::from(&headers.service),
            id: headers.id.clone(),
            user_id: headers.user_id.clone(),
            issue: headers.issue.clone(),
            protocol_version: headers.protocol_version.clone(),
            represented_party: headers.represented_party.clone(),
        }
    }

    pub fn to_xroad_headers(&self) -> XRoadResult<XRoadHeaders> {
        let client = ClientId::try_from(self.client.clone())?;
        let service = ServiceId::try_from(self.service.clone())?;
        Ok(XRoadHeaders {
            client,
            service,
            id: self.id.clone(),
            user_id: self.user_id.clone(),
            issue: self.issue.clone(),
            protocol_version: self.protocol_version.clone(),
            represented_party: self.represented_party.clone(),
        })
    }
}

/// SOAP Fault Detail element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SoapFaultDetailXml {
    #[serde(
        rename = "xroad:faultDetail",
        alias = "faultDetail",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fault_detail: Option<String>,
}

/// Standard SOAP 1.1 Fault element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoapFaultXml {
    #[serde(rename = "faultcode")]
    pub fault_code: String,
    #[serde(rename = "faultstring")]
    pub fault_string: String,
    #[serde(
        rename = "faultactor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fault_actor: Option<String>,
    #[serde(rename = "detail", default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<SoapFaultDetailXml>,
}

/// Generic SOAP Body container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SoapBodyXml {
    #[serde(
        rename = "SOAP-ENV:Fault",
        alias = "Fault",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fault: Option<SoapFaultXml>,
}

/// Complete X-Road SOAP 1.1 Envelope (`<SOAP-ENV:Envelope>`) serialized & deserialized via quick-xml serde.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "Envelope")]
pub struct SoapEnvelopeXml {
    #[serde(
        rename = "@xmlns:SOAP-ENV",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub xmlns_soap_env: Option<String>,
    #[serde(
        rename = "@xmlns:xroad",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub xmlns_xroad: Option<String>,
    #[serde(rename = "@xmlns:id", default, skip_serializing_if = "Option::is_none")]
    pub xmlns_id: Option<String>,
    #[serde(rename = "SOAP-ENV:Header", alias = "Header")]
    pub header: SoapHeaderXml,
    #[serde(rename = "SOAP-ENV:Body", alias = "Body")]
    pub body: SoapBodyXml,
    #[serde(skip)]
    pub payload: Option<String>,
}

impl SoapEnvelopeXml {
    /// Creates a new SOAP request envelope.
    pub fn new_request(client: ClientId, service: ServiceId, payload: Option<String>) -> Self {
        let headers = XRoadHeaders::new(client, service);
        Self::from_xroad_headers(&headers, payload)
    }

    /// Creates a new SOAP response envelope.
    pub fn new_response(
        client: ClientId,
        service: ServiceId,
        query_id: String,
        payload: Option<String>,
    ) -> Self {
        let mut headers = XRoadHeaders::new(client, service);
        headers.id = query_id;
        Self::from_xroad_headers(&headers, payload)
    }

    /// Creates a SOAP envelope from `XRoadHeaders` and optional body payload.
    pub fn from_xroad_headers(headers: &XRoadHeaders, payload: Option<String>) -> Self {
        Self {
            xmlns_soap_env: Some(SOAP_ENV_NS.to_string()),
            xmlns_xroad: Some(XROAD_NS.to_string()),
            xmlns_id: Some(XROAD_ID_NS.to_string()),
            header: SoapHeaderXml::from_xroad_headers(headers),
            body: SoapBodyXml { fault: None },
            payload,
        }
    }

    /// Creates a standard X-Road SOAP Fault envelope.
    pub fn new_fault(
        headers: Option<&XRoadHeaders>,
        fault_code: &str,
        fault_string: &str,
        fault_detail: Option<&str>,
    ) -> Self {
        let default_client = ClientId::member("XX", "COM", "UNKNOWN").unwrap();
        let default_service =
            ServiceId::new(default_client.clone(), "unknown", None::<&str>).unwrap();
        let header = match headers {
            Some(h) => SoapHeaderXml::from_xroad_headers(h),
            None => SoapHeaderXml {
                client: XRoadClientIdentifierType::from(&default_client),
                service: XRoadServiceIdentifierType::from(&default_service),
                id: uuid::Uuid::new_v4().to_string(),
                user_id: None,
                issue: None,
                protocol_version: XROAD_PROTOCOL_VERSION.to_string(),
                represented_party: None,
            },
        };

        Self {
            xmlns_soap_env: Some(SOAP_ENV_NS.to_string()),
            xmlns_xroad: Some(XROAD_NS.to_string()),
            xmlns_id: Some(XROAD_ID_NS.to_string()),
            header,
            body: SoapBodyXml {
                fault: Some(SoapFaultXml {
                    fault_code: fault_code.to_string(),
                    fault_string: fault_string.to_string(),
                    fault_actor: None,
                    detail: fault_detail.map(|d| SoapFaultDetailXml {
                        fault_detail: Some(d.to_string()),
                    }),
                }),
            },
            payload: None,
        }
    }

    /// Serializes this envelope into an XML string using quick-xml serde.
    pub fn to_xml(&self) -> XRoadResult<String> {
        let mut body = quick_xml::se::to_string(self).map_err(|e| {
            XRoadError::MessageProtocol(format!("Failed to serialize SOAP envelope XML: {e}"))
        })?;
        if body.starts_with("<Envelope") {
            body = body
                .replacen("<Envelope", "<SOAP-ENV:Envelope", 1)
                .replacen("</Envelope>", "</SOAP-ENV:Envelope>", 1);
        }
        if let Some(p) = &self.payload {
            if body.contains("<SOAP-ENV:Body/>") {
                body = body.replacen(
                    "<SOAP-ENV:Body/>",
                    &format!("<SOAP-ENV:Body>{}</SOAP-ENV:Body>", p),
                    1,
                );
            } else if body.contains("<SOAP-ENV:Body></SOAP-ENV:Body>") {
                body = body.replacen(
                    "<SOAP-ENV:Body></SOAP-ENV:Body>",
                    &format!("<SOAP-ENV:Body>{}</SOAP-ENV:Body>", p),
                    1,
                );
            }
        }
        Ok(format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n{}",
            body
        ))
    }

    /// Parses a SOAP envelope from an XML string using quick-xml serde.
    pub fn from_xml(xml: &str) -> XRoadResult<Self> {
        quick_xml::de::from_str(xml).map_err(|e| {
            XRoadError::MessageProtocol(format!("Failed to parse SOAP envelope XML: {e}"))
        })
    }

    /// Returns whether this envelope is a SOAP Fault.
    pub fn is_fault(&self) -> bool {
        self.body.fault.is_some()
    }

    /// Extracts XRoadHeaders from this envelope.
    pub fn to_xroad_headers(&self) -> XRoadResult<XRoadHeaders> {
        self.header.to_xroad_headers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soap_envelope_serde_roundtrip() {
        let client = ClientId::member("EE", "BUSINESS", "consumer").unwrap();
        let service = ServiceId::from_parts(
            "EE",
            "BUSINESS",
            "producer",
            None::<&str>,
            "testQuery",
            None::<&str>,
        )
        .unwrap();
        let envelope = SoapEnvelopeXml::new_request(
            client.clone(),
            service.clone(),
            Some("<testQuery><name>foo</name></testQuery>".to_string()),
        );

        let xml = envelope.to_xml().expect("Should serialize SOAP XML");
        assert!(xml.contains("SOAP-ENV:Envelope"));
        assert!(xml.contains("xroad:client"));
        assert!(xml.contains("xroad:service"));
        assert!(xml.contains("testQuery"));

        let parsed = SoapEnvelopeXml::from_xml(&xml).expect("Should parse with quick-xml serde");
        assert_eq!(parsed.header.client.member_code, "consumer");
        assert_eq!(parsed.header.service.member_code, "producer");
        assert_eq!(parsed.header.service.service_code, "testQuery");
        assert!(!parsed.is_fault());
    }

    #[test]
    fn test_soap_fault_serde() {
        let fault = SoapEnvelopeXml::new_fault(
            None,
            "SOAP-ENV:Server",
            "Server.ServerProxy.ServiceNotFound",
            Some("Service not found in local service database"),
        );
        let xml = fault.to_xml().unwrap();
        assert!(xml.contains("SOAP-ENV:Fault"));
        assert!(xml.contains("Server.ServerProxy.ServiceNotFound"));

        let parsed = SoapEnvelopeXml::from_xml(&xml).unwrap();
        assert!(parsed.is_fault());
        let f = parsed.body.fault.unwrap();
        assert_eq!(f.fault_code, "SOAP-ENV:Server");
        assert_eq!(f.fault_string, "Server.ServerProxy.ServiceNotFound");
        assert_eq!(
            f.detail.unwrap().fault_detail.unwrap(),
            "Service not found in local service database"
        );
    }

    #[test]
    fn test_parse_sample_xroad_query() {
        let sample = r#"<?xml version="1.0" encoding="utf-8"?>
<SOAP-ENV:Envelope
        xmlns:SOAP-ENV="http://schemas.xmlsoap.org/soap/envelope/"
        xmlns:xroad="http://x-road.eu/xsd/xroad.xsd"
        xmlns:id="http://x-road.eu/xsd/identifiers">
    <SOAP-ENV:Header>
        <xroad:client id:objectType="MEMBER">
            <id:xRoadInstance>EE</id:xRoadInstance>
            <id:memberClass>BUSINESS</id:memberClass>
            <id:memberCode>consumer</id:memberCode>
        </xroad:client>
        <xroad:service id:objectType="SERVICE">
            <id:xRoadInstance>EE</id:xRoadInstance>
            <id:memberClass>BUSINESS</id:memberClass>
            <id:memberCode>producer</id:memberCode>
            <id:serviceCode>testQuery</id:serviceCode>
        </xroad:service>
        <xroad:userId>EE37702211234</xroad:userId>
        <xroad:id>1234567890</xroad:id>
        <xroad:protocolVersion>4.0</xroad:protocolVersion>
    </SOAP-ENV:Header>
    <SOAP-ENV:Body>
        <ns1:testQuery xmlns:ns1="http://cosumer.ee.xroad.net">
            <request>
                <name>foo</name>
                <age>35</age>
            </request>
        </ns1:testQuery>
    </SOAP-ENV:Body>
</SOAP-ENV:Envelope>"#;

        let parsed = SoapEnvelopeXml::from_xml(sample).expect("Must parse canonical X-Road query");
        let headers = parsed.to_xroad_headers().unwrap();
        assert_eq!(headers.client.member_code, "consumer");
        assert_eq!(headers.service.service_code, "testQuery");
        assert_eq!(headers.id, "1234567890");
        assert_eq!(headers.user_id, Some("EE37702211234".to_string()));
    }
}
