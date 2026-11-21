use base64::prelude::*;
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use serde::{Deserialize, Serialize};

/// XML DTO for the root `<conf>` / `<ns3:conf>` element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename = "conf")]
pub struct SharedParamsXml {
    #[serde(rename = "@xmlns:ns2", default)]
    pub ns2: Option<String>,
    #[serde(rename = "@xmlns:ns3", default)]
    pub ns3: Option<String>,
    #[serde(rename = "instanceIdentifier")]
    pub instance_identifier: String,
    #[serde(rename = "approvedCA", default)]
    pub approved_cas: Vec<ApprovedCaXml>,
    #[serde(rename = "approvedTSA", default)]
    pub approved_tsas: Vec<ApprovedTsaXml>,
    #[serde(rename = "member", default)]
    pub members: Vec<MemberXml>,
    #[serde(rename = "securityServer", default)]
    pub security_servers: Vec<SecurityServerXml>,
    #[serde(rename = "globalGroup", default, skip_serializing_if = "Vec::is_empty")]
    pub global_groups: Vec<GlobalGroupXml>,
    #[serde(rename = "globalSettings", default, skip_serializing_if = "Option::is_none")]
    pub global_settings: Option<GlobalSettingsXml>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovedCaXml {
    pub name: String,
    #[serde(rename = "authenticationOnly", default)]
    pub authentication_only: bool,
    #[serde(rename = "topCA")]
    pub top_ca: TopCaXml,
    #[serde(rename = "certificateProfileInfo", default, skip_serializing_if = "Option::is_none")]
    pub certificate_profile_info: Option<String>,
    #[serde(rename = "acmeServer", default, skip_serializing_if = "Option::is_none")]
    pub acme_server: Option<AcmeServerXml>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopCaXml {
    pub cert: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ocsp: Option<OcspXml>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OcspXml {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cert: Option<String>,
    #[serde(rename = "costType", default, skip_serializing_if = "Option::is_none")]
    pub cost_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcmeServerXml {
    #[serde(rename = "directoryURL", default, skip_serializing_if = "Option::is_none")]
    pub directory_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovedTsaXml {
    pub name: String,
    pub url: String,
    pub cert: String,
    #[serde(rename = "costType", default, skip_serializing_if = "Option::is_none")]
    pub cost_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemberXml {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "memberClass")]
    pub member_class: MemberClassXml,
    #[serde(rename = "memberCode")]
    pub member_code: String,
    pub name: String,
    #[serde(rename = "subsystem", default, skip_serializing_if = "Vec::is_empty")]
    pub subsystems: Vec<SubsystemXml>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemberClassXml {
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubsystemXml {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "subsystemCode")]
    pub subsystem_code: String,
    #[serde(rename = "subsystemName", default, skip_serializing_if = "Option::is_none")]
    pub subsystem_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityServerXml {
    pub owner: String,
    #[serde(rename = "serverCode")]
    pub server_code: String,
    pub address: String,
    #[serde(rename = "authCertHash", default, skip_serializing_if = "Vec::is_empty")]
    pub auth_cert_hashes: Vec<String>,
    #[serde(rename = "client", default, skip_serializing_if = "Vec::is_empty")]
    pub clients: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalGroupXml {
    #[serde(rename = "groupCode")]
    pub group_code: String,
    pub description: String,
    #[serde(rename = "groupMember", default, skip_serializing_if = "Vec::is_empty")]
    pub group_members: Vec<GroupMemberXml>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupMemberXml {
    #[serde(rename = "@objectType", alias = "@ns2:objectType", default, skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    #[serde(rename = "xRoadInstance", alias = "ns2:xRoadInstance")]
    pub xroad_instance: String,
    #[serde(rename = "memberClass", alias = "ns2:memberClass")]
    pub member_class: String,
    #[serde(rename = "memberCode", alias = "ns2:memberCode")]
    pub member_code: String,
    #[serde(rename = "subsystemCode", alias = "ns2:subsystemCode", default, skip_serializing_if = "Option::is_none")]
    pub subsystem_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalSettingsXml {
    #[serde(rename = "ocspFreshnessSeconds", default, skip_serializing_if = "Option::is_none")]
    pub ocsp_freshness_seconds: Option<u64>,
}

/// Model representing an Approved Certification Authority in shared-params.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedApprovedCa {
    pub name: String,
    pub authentication_only: bool,
    pub top_ca_cert: Vec<u8>,
    pub ocsp_url: Option<String>,
    pub ocsp_cert: Option<Vec<u8>>,
    pub certificate_profile_info: Option<String>,
    pub acme_server_url: Option<String>,
}

/// Model representing an Approved Time-Stamping Authority in shared-params.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedApprovedTsa {
    pub name: String,
    pub url: String,
    pub cert: Vec<u8>,
}

/// Model representing a Subsystem inside a Member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSubsystem {
    pub id: String,
    pub subsystem_code: String,
    pub subsystem_name: Option<String>,
}

/// Model representing a Member in shared-params.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedMember {
    pub id: String,
    pub member_class: String,
    pub member_code: String,
    pub name: String,
    pub subsystems: Vec<SharedSubsystem>,
}

/// Model representing a Security Server in shared-params.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedSecurityServer {
    pub owner_id: String,
    pub server_code: String,
    pub address: String,
    pub auth_cert_hashes: Vec<String>,
    pub client_ids: Vec<String>,
}

/// Model representing a Global Group in shared-params.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedGlobalGroup {
    pub group_code: String,
    pub description: String,
    pub members: Vec<ClientId>,
}

/// Complete in-memory domain representation of `shared-params.xml`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SharedParams {
    pub instance_identifier: String,
    pub approved_cas: Vec<SharedApprovedCa>,
    pub approved_tsas: Vec<SharedApprovedTsa>,
    pub members: Vec<SharedMember>,
    pub security_servers: Vec<SharedSecurityServer>,
    pub global_groups: Vec<SharedGlobalGroup>,
    pub ocsp_freshness_seconds: Option<u64>,
}

impl SharedParams {
    /// Parses a `shared-params.xml` string using `quick-xml`'s serde deserializer.
    pub fn parse_xml(xml: &str) -> XRoadResult<Self> {
        let dto: SharedParamsXml = quick_xml::de::from_str(xml)
            .map_err(|e| XRoadError::Configuration(format!("quick-xml serde deserialization error: {e}")))?;

        let approved_cas = dto
            .approved_cas
            .into_iter()
            .map(|ca| {
                let top_ca_cert = decode_b64(&ca.top_ca.cert);
                let (ocsp_url, ocsp_cert) = match ca.top_ca.ocsp {
                    Some(ocsp) => (Some(ocsp.url), ocsp.cert.map(|c| decode_b64(&c))),
                    None => (None, None),
                };

                SharedApprovedCa {
                    name: ca.name,
                    authentication_only: ca.authentication_only,
                    top_ca_cert,
                    ocsp_url,
                    ocsp_cert,
                    certificate_profile_info: ca.certificate_profile_info,
                    acme_server_url: ca.acme_server.and_then(|a| a.directory_url),
                }
            })
            .collect();

        let approved_tsas = dto
            .approved_tsas
            .into_iter()
            .map(|tsa| SharedApprovedTsa {
                name: tsa.name,
                url: tsa.url,
                cert: decode_b64(&tsa.cert),
            })
            .collect();

        let members = dto
            .members
            .into_iter()
            .map(|m| SharedMember {
                id: m.id,
                member_class: m.member_class.code,
                member_code: m.member_code,
                name: m.name,
                subsystems: m
                    .subsystems
                    .into_iter()
                    .map(|s| SharedSubsystem {
                        id: s.id,
                        subsystem_code: s.subsystem_code,
                        subsystem_name: s.subsystem_name,
                    })
                    .collect(),
            })
            .collect();

        let security_servers = dto
            .security_servers
            .into_iter()
            .map(|ss| SharedSecurityServer {
                owner_id: ss.owner,
                server_code: ss.server_code,
                address: ss.address,
                auth_cert_hashes: ss.auth_cert_hashes,
                client_ids: ss.clients,
            })
            .collect();

        let global_groups = dto
            .global_groups
            .into_iter()
            .map(|g| {
                let members = g
                    .group_members
                    .into_iter()
                    .map(|gm| {
                        if let Some(sub) = gm.subsystem_code {
                            ClientId::subsystem_unchecked(
                                gm.xroad_instance,
                                gm.member_class,
                                gm.member_code,
                                sub,
                            )
                        } else {
                            ClientId::member_unchecked(
                                gm.xroad_instance,
                                gm.member_class,
                                gm.member_code,
                            )
                        }
                    })
                    .collect();

                SharedGlobalGroup {
                    group_code: g.group_code,
                    description: g.description,
                    members,
                }
            })
            .collect();

        let ocsp_freshness_seconds = dto
            .global_settings
            .and_then(|gs| gs.ocsp_freshness_seconds);

        Ok(SharedParams {
            instance_identifier: dto.instance_identifier,
            approved_cas,
            approved_tsas,
            members,
            security_servers,
            global_groups,
            ocsp_freshness_seconds,
        })
    }

    /// Serializes `SharedParams` into XML using `quick-xml`'s serde serializer.
    pub fn to_xml(&self) -> String {
        let dto = SharedParamsXml {
            ns2: Some("http://x-road.eu/xsd/identifiers".to_string()),
            ns3: Some("http://x-road.eu/xsd/xroad.xsd".to_string()),
            instance_identifier: self.instance_identifier.clone(),
            approved_cas: self
                .approved_cas
                .iter()
                .map(|ca| ApprovedCaXml {
                    name: ca.name.clone(),
                    authentication_only: ca.authentication_only,
                    top_ca: TopCaXml {
                        cert: BASE64_STANDARD.encode(&ca.top_ca_cert),
                        ocsp: ca.ocsp_url.as_ref().map(|url| OcspXml {
                            url: url.clone(),
                            cert: ca.ocsp_cert.as_ref().map(|c| BASE64_STANDARD.encode(c)),
                            cost_type: Some("FREE".to_string()),
                        }),
                    },
                    certificate_profile_info: ca.certificate_profile_info.clone(),
                    acme_server: ca.acme_server_url.as_ref().map(|url| AcmeServerXml {
                        directory_url: Some(url.clone()),
                    }),
                })
                .collect(),
            approved_tsas: self
                .approved_tsas
                .iter()
                .map(|tsa| ApprovedTsaXml {
                    name: tsa.name.clone(),
                    url: tsa.url.clone(),
                    cert: BASE64_STANDARD.encode(&tsa.cert),
                    cost_type: Some("FREE".to_string()),
                })
                .collect(),
            members: self
                .members
                .iter()
                .map(|m| MemberXml {
                    id: m.id.clone(),
                    member_class: MemberClassXml {
                        code: m.member_class.clone(),
                        description: None,
                    },
                    member_code: m.member_code.clone(),
                    name: m.name.clone(),
                    subsystems: m
                        .subsystems
                        .iter()
                        .map(|s| SubsystemXml {
                            id: s.id.clone(),
                            subsystem_code: s.subsystem_code.clone(),
                            subsystem_name: s.subsystem_name.clone(),
                        })
                        .collect(),
                })
                .collect(),
            security_servers: self
                .security_servers
                .iter()
                .map(|ss| SecurityServerXml {
                    owner: ss.owner_id.clone(),
                    server_code: ss.server_code.clone(),
                    address: ss.address.clone(),
                    auth_cert_hashes: ss.auth_cert_hashes.clone(),
                    clients: ss.client_ids.clone(),
                })
                .collect(),
            global_groups: self
                .global_groups
                .iter()
                .map(|g| GlobalGroupXml {
                    group_code: g.group_code.clone(),
                    description: g.description.clone(),
                    group_members: g
                        .members
                        .iter()
                        .map(|m| GroupMemberXml {
                            object_type: Some(
                                if m.subsystem_code.is_some() {
                                    "SUBSYSTEM".to_string()
                                } else {
                                    "MEMBER".to_string()
                                },
                            ),
                            xroad_instance: m.xroad_instance.clone(),
                            member_class: m.member_class.clone(),
                            member_code: m.member_code.clone(),
                            subsystem_code: m.subsystem_code.clone(),
                        })
                        .collect(),
                })
                .collect(),
            global_settings: self
                .ocsp_freshness_seconds
                .map(|sec| GlobalSettingsXml {
                    ocsp_freshness_seconds: Some(sec),
                }),
        };

        let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        let body = quick_xml::se::to_string(&dto).unwrap_or_default();
        out.push_str(&body);
        out
    }
}

fn decode_b64(s: &str) -> Vec<u8> {
    let clean = s.replace(['\r', '\n', ' '], "");
    BASE64_STANDARD.decode(&clean).unwrap_or_else(|_| s.as_bytes().to_vec())
}
