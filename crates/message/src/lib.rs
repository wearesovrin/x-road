use common::{XROAD_PROTOCOL_VERSION, XRoadError, XRoadResult, headers};
use identifiers::{ClientId, ServiceId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Standard headers included in every X-Road message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XRoadHeaders {
    pub client: ClientId,
    pub service: ServiceId,
    pub id: String,
    pub user_id: Option<String>,
    pub issue: Option<String>,
    pub protocol_version: String,
    pub represented_party: Option<String>,
}

impl XRoadHeaders {
    pub fn new(client: ClientId, service: ServiceId) -> Self {
        Self {
            client,
            service,
            id: Uuid::new_v4().to_string(),
            user_id: None,
            issue: None,
            protocol_version: XROAD_PROTOCOL_VERSION.to_string(),
            represented_party: None,
        }
    }

    /// Parses X-Road headers from a map of HTTP headers (case-insensitive keys).
    pub fn from_http_headers(headers_map: &HashMap<String, String>) -> XRoadResult<Self> {
        let get_header = |name: &str| -> Option<&String> {
            headers_map
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v)
        };

        let client_str = get_header(headers::X_ROAD_CLIENT).ok_or_else(|| {
            XRoadError::MessageProtocol("Missing x-road-client header".to_string())
        })?;
        let service_str = get_header(headers::X_ROAD_SERVICE).ok_or_else(|| {
            XRoadError::MessageProtocol("Missing x-road-service header".to_string())
        })?;

        let client: ClientId = client_str.parse()?;
        let service: ServiceId = service_str.parse()?;

        let id = get_header(headers::X_ROAD_ID)
            .cloned()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let user_id = get_header(headers::X_ROAD_USER_ID).cloned();
        let issue = get_header(headers::X_ROAD_ISSUE).cloned();
        let protocol_version = get_header(headers::X_ROAD_PROTOCOL_VERSION)
            .cloned()
            .unwrap_or_else(|| XROAD_PROTOCOL_VERSION.to_string());
        let represented_party = get_header(headers::X_ROAD_REPRESENTED_PARTY).cloned();

        Ok(Self {
            client,
            service,
            id,
            user_id,
            issue,
            protocol_version,
            represented_party,
        })
    }

    /// Serializes X-Road headers into key-value pairs suitable for HTTP request headers.
    pub fn to_http_headers(&self) -> Vec<(&'static str, String)> {
        let mut list = vec![
            (headers::X_ROAD_CLIENT, self.client.to_string()),
            (headers::X_ROAD_SERVICE, self.service.to_string()),
            (headers::X_ROAD_ID, self.id.clone()),
            (
                headers::X_ROAD_PROTOCOL_VERSION,
                self.protocol_version.clone(),
            ),
        ];
        if let Some(user) = &self.user_id {
            list.push((headers::X_ROAD_USER_ID, user.clone()));
        }
        if let Some(iss) = &self.issue {
            list.push((headers::X_ROAD_ISSUE, iss.clone()));
        }
        if let Some(rep) = &self.represented_party {
            list.push((headers::X_ROAD_REPRESENTED_PARTY, rep.clone()));
        }
        list
    }
}

/// MIME part descriptors for X-Road Transport Messages.
#[derive(Debug, Clone)]
pub enum MimePart {
    OcspResponse(Vec<u8>),
    RestRequestHeader(Vec<u8>),
    RestRequestBody(Vec<u8>),
    SoapBody(Vec<u8>),
    Attachment { content_type: String, data: Vec<u8> },
    Signature(Vec<u8>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headers_roundtrip() {
        let client = ClientId::subsystem("DEV", "COM", "111", "sub1");
        let service = ServiceId::new(client.clone(), "testService", Some("v1"));
        let headers = XRoadHeaders::new(client, service);

        let http_headers = headers.to_http_headers();
        let map: HashMap<String, String> = http_headers
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        let parsed = XRoadHeaders::from_http_headers(&map).unwrap();
        assert_eq!(headers.client, parsed.client);
        assert_eq!(headers.service, parsed.service);
        assert_eq!(headers.id, parsed.id);
    }
}
