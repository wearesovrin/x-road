pub mod shared_params;

use chrono::{DateTime, Utc};
use common::XRoadResult;
use identifiers::{ClientId, SecurityServerId};
use serde::{Deserialize, Serialize};
pub use shared_params::*;
use std::collections::{HashMap, HashSet};

/// Configuration anchor pointing to the source of global configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationAnchor {
    pub instance_identifier: String,
    pub generated_at: DateTime<Utc>,
    pub download_urls: Vec<String>,
    pub verification_cert_hashes: Vec<String>,
}

/// Approved Certification Authority in global configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovedCa {
    pub name: String,
    pub top_ca_cert: Vec<u8>,
    pub intermediate_ca_certs: Vec<Vec<u8>>,
    pub ocsp_response_freshness: u64,
}

/// Approved Time-Stamping Provider in global configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovedTsp {
    pub name: String,
    pub url: String,
    pub cert: Vec<u8>,
}

/// Detailed information about a registered Security Server.
#[derive(Debug, Clone, Default)]
pub struct SecurityServerInfo {
    pub address: String,
    pub auth_cert_hashes: Vec<String>,
    pub clients: HashSet<ClientId>,
}

/// Global group details.
#[derive(Debug, Clone, Default)]
pub struct GlobalGroupInfo {
    pub description: String,
    pub members: HashSet<ClientId>,
}

/// In-memory cache of global configuration data.
#[derive(Debug, Default, Clone)]
pub struct GlobalConf {
    pub instance_identifier: String,
    pub members: HashMap<ClientId, String>,
    pub security_servers: HashMap<SecurityServerId, String>,
    pub client_to_server: HashMap<ClientId, SecurityServerId>,
    pub sec_server_details: HashMap<SecurityServerId, SecurityServerInfo>,
    pub approved_cas: Vec<ApprovedCa>,
    pub approved_tsps: Vec<ApprovedTsp>,
    pub global_groups: HashMap<String, GlobalGroupInfo>,
    pub ocsp_freshness_seconds: u64,
}

impl GlobalConf {
    pub fn new(instance_identifier: impl Into<String>) -> Self {
        Self {
            instance_identifier: instance_identifier.into(),
            ocsp_freshness_seconds: 3600,
            ..Default::default()
        }
    }

    /// Creates a `GlobalConf` cache by parsing a `shared-params.xml` string.
    pub fn from_shared_params_xml(xml: &str) -> XRoadResult<Self> {
        let params = SharedParams::parse_xml(xml)?;
        Ok(Self::from_shared_params(&params))
    }

    /// Constructs a `GlobalConf` from the parsed `SharedParams` model.
    pub fn from_shared_params(params: &SharedParams) -> Self {
        let instance = &params.instance_identifier;
        let mut members = HashMap::new();
        let mut id_to_client: HashMap<String, ClientId> = HashMap::new();
        let mut client_to_member_id: HashMap<String, ClientId> = HashMap::new();

        // 1. Process members & subsystems
        for m in &params.members {
            let member_client =
                ClientId::member_unchecked(instance, &m.member_class, &m.member_code);
            members.insert(member_client.clone(), m.name.clone());
            id_to_client.insert(m.id.clone(), member_client.clone());
            client_to_member_id.insert(m.id.clone(), member_client.clone());

            for sub in &m.subsystems {
                let sub_client = ClientId::subsystem_unchecked(
                    instance,
                    &m.member_class,
                    &m.member_code,
                    &sub.subsystem_code,
                );
                let sub_name = sub
                    .subsystem_name
                    .clone()
                    .unwrap_or_else(|| format!("{}/{}", m.name, sub.subsystem_code));
                members.insert(sub_client.clone(), sub_name);
                id_to_client.insert(sub.id.clone(), sub_client);
            }
        }

        // 2. Process security servers
        let mut security_servers = HashMap::new();
        let mut client_to_server = HashMap::new();
        let mut sec_server_details = HashMap::new();

        for ss in &params.security_servers {
            if let Some(owner_client) = client_to_member_id.get(&ss.owner_id) {
                let server_id = match SecurityServerId::from_owner(owner_client, &ss.server_code) {
                    Ok(id) => id,
                    Err(_) => continue,
                };

                security_servers.insert(server_id.clone(), ss.address.clone());

                let mut client_set = HashSet::new();
                for client_ref in &ss.client_ids {
                    if let Some(c_id) = id_to_client.get(client_ref) {
                        client_to_server.insert(c_id.clone(), server_id.clone());
                        client_set.insert(c_id.clone());
                    }
                }

                sec_server_details.insert(
                    server_id,
                    SecurityServerInfo {
                        address: ss.address.clone(),
                        auth_cert_hashes: ss.auth_cert_hashes.clone(),
                        clients: client_set,
                    },
                );
            }
        }

        // 3. Approved CAs & TSAs
        let approved_cas: Vec<ApprovedCa> = params
            .approved_cas
            .iter()
            .map(|ca| ApprovedCa {
                name: ca.name.clone(),
                top_ca_cert: ca.top_ca_cert.clone(),
                intermediate_ca_certs: vec![],
                ocsp_response_freshness: params.ocsp_freshness_seconds.unwrap_or(3600),
            })
            .collect();

        let approved_tsps: Vec<ApprovedTsp> = params
            .approved_tsas
            .iter()
            .map(|tsa| ApprovedTsp {
                name: tsa.name.clone(),
                url: tsa.url.clone(),
                cert: tsa.cert.clone(),
            })
            .collect();

        // 4. Global Groups
        let mut global_groups = HashMap::new();
        for g in &params.global_groups {
            let members_set: HashSet<ClientId> = g.members.iter().cloned().collect();
            global_groups.insert(
                g.group_code.clone(),
                GlobalGroupInfo {
                    description: g.description.clone(),
                    members: members_set,
                },
            );
        }

        GlobalConf {
            instance_identifier: instance.clone(),
            members,
            security_servers,
            client_to_server,
            sec_server_details,
            approved_cas,
            approved_tsps,
            global_groups,
            ocsp_freshness_seconds: params.ocsp_freshness_seconds.unwrap_or(3600),
        }
    }

    /// Checks if a client is registered in the global configuration.
    pub fn is_member_registered(&self, client_id: &ClientId) -> bool {
        self.members.contains_key(client_id)
    }

    /// Resolves the address (host:port) for a given Security Server ID.
    pub fn get_security_server_address(&self, server_id: &SecurityServerId) -> Option<&String> {
        self.security_servers.get(server_id)
    }

    /// Registers a Security Server address in the configuration.
    pub fn register_security_server(
        &mut self,
        server_id: SecurityServerId,
        address: impl Into<String>,
    ) {
        self.security_servers.insert(server_id, address.into());
    }

    /// Links a client to a Security Server in the configuration.
    pub fn link_client_to_security_server(
        &mut self,
        client_id: ClientId,
        server_id: SecurityServerId,
    ) {
        self.client_to_server.insert(client_id, server_id);
    }

    /// Resolves which Security Server hosts the given client.
    pub fn get_security_server_for_client(
        &self,
        client_id: &ClientId,
    ) -> Option<&SecurityServerId> {
        if let Some(ss_id) = self.client_to_server.get(client_id) {
            return Some(ss_id);
        }
        // Fall back to owner member if subsystem is queried
        if client_id.subsystem_code.is_some() {
            let member_id = client_id.member_id();
            if let Some(ss_id) = self.client_to_server.get(&member_id) {
                return Some(ss_id);
            }
        }
        None
    }

    /// Resolves the network address of the Security Server hosting the given client.
    pub fn get_security_server_address_for_client(&self, client_id: &ClientId) -> Option<&String> {
        let server_id = self.get_security_server_for_client(client_id)?;
        self.security_servers.get(server_id)
    }

    /// Checks if a client belongs to the given global group.
    pub fn is_member_in_group(&self, client_id: &ClientId, group_code: &str) -> bool {
        if let Some(g) = self.global_groups.get(group_code) {
            if g.members.contains(client_id) {
                return true;
            }
            if client_id.subsystem_code.is_some() {
                let member_id = client_id.member_id();
                return g.members.contains(&member_id);
            }
        }
        false
    }

    /// Returns all Approved CAs in the configuration.
    pub fn get_approved_cas(&self) -> &[ApprovedCa] {
        &self.approved_cas
    }

    /// Returns all Approved TSAs in the configuration.
    pub fn get_approved_tsps(&self) -> &[ApprovedTsp] {
        &self.approved_tsps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_query_shared_params() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<ns3:conf xmlns:ns2="http://x-road.eu/xsd/identifiers" xmlns:ns3="http://x-road.eu/xsd/xroad.xsd">
    <instanceIdentifier>DEV</instanceIdentifier>
    <approvedCA>
        <name>Test CA</name>
        <authenticationOnly>false</authenticationOnly>
        <topCA>
            <cert>dGVzdC1jYS1jZXJ0</cert>
            <ocsp>
                <url>http://testca:8888</url>
                <costType>FREE</costType>
            </ocsp>
        </topCA>
    </approvedCA>
    <approvedTSA>
        <name>Test TSA</name>
        <url>http://testca:8899</url>
        <cert>dGVzdC10c2EtY2VydA==</cert>
    </approvedTSA>
    <member id="id0">
        <memberClass>
            <code>COM</code>
        </memberClass>
        <memberCode>1234</memberCode>
        <name>Acme Corp</name>
        <subsystem id="id1">
            <subsystemCode>MANAGEMENT</subsystemCode>
            <subsystemName>Management Services</subsystemName>
        </subsystem>
    </member>
    <securityServer>
        <owner>id0</owner>
        <serverCode>SS1</serverCode>
        <address>ss1.acme.com:8443</address>
        <authCertHash>abc123hash</authCertHash>
        <client>id1</client>
    </securityServer>
    <globalGroup>
        <groupCode>vip-members</groupCode>
        <description>VIP Members</description>
        <groupMember ns2:objectType="SUBSYSTEM">
            <ns2:xRoadInstance>DEV</ns2:xRoadInstance>
            <ns2:memberClass>COM</ns2:memberClass>
            <ns2:memberCode>1234</ns2:memberCode>
            <ns2:subsystemCode>MANAGEMENT</ns2:subsystemCode>
        </groupMember>
    </globalGroup>
    <globalSettings>
        <ocspFreshnessSeconds>1800</ocspFreshnessSeconds>
    </globalSettings>
</ns3:conf>"#;

        let conf =
            GlobalConf::from_shared_params_xml(xml).expect("Failed to parse shared-params.xml");
        assert_eq!(conf.instance_identifier, "DEV");
        assert_eq!(conf.ocsp_freshness_seconds, 1800);

        // Verify member lookup
        let member_id = ClientId::member_unchecked("DEV", "COM", "1234");
        let sub_id = ClientId::subsystem_unchecked("DEV", "COM", "1234", "MANAGEMENT");
        assert!(conf.is_member_registered(&member_id));
        assert!(conf.is_member_registered(&sub_id));

        // Verify security server lookup
        let sec_server_id = SecurityServerId::new("DEV", "COM", "1234", "SS1").unwrap();
        assert_eq!(
            conf.get_security_server_address(&sec_server_id),
            Some(&"ss1.acme.com:8443".to_string())
        );

        // Client to security server lookup
        assert_eq!(
            conf.get_security_server_for_client(&sub_id),
            Some(&sec_server_id)
        );
        assert_eq!(
            conf.get_security_server_address_for_client(&sub_id),
            Some(&"ss1.acme.com:8443".to_string())
        );

        // CA & TSA
        assert_eq!(conf.approved_cas.len(), 1);
        assert_eq!(conf.approved_cas[0].name, "Test CA");
        assert_eq!(conf.approved_cas[0].top_ca_cert, b"test-ca-cert");

        assert_eq!(conf.approved_tsps.len(), 1);
        assert_eq!(conf.approved_tsps[0].name, "Test TSA");
        assert_eq!(conf.approved_tsps[0].cert, b"test-tsa-cert");

        // Group check
        assert!(conf.is_member_in_group(&sub_id, "vip-members"));
        let unknown = ClientId::member_unchecked("DEV", "COM", "9999");
        assert!(!conf.is_member_in_group(&unknown, "vip-members"));
    }

    #[test]
    fn test_shared_params_roundtrip() {
        let params = SharedParams {
            instance_identifier: "DEV".to_string(),
            approved_cas: vec![SharedApprovedCa {
                name: "Roundtrip CA".to_string(),
                authentication_only: false,
                top_ca_cert: b"top-ca-data".to_vec(),
                ocsp_url: Some("http://ocsp.example.com".to_string()),
                ocsp_cert: None,
                certificate_profile_info: None,
                acme_server_url: None,
            }],
            approved_tsas: vec![SharedApprovedTsa {
                name: "Roundtrip TSA".to_string(),
                url: "http://tsa.example.com".to_string(),
                cert: b"tsa-cert-data".to_vec(),
            }],
            members: vec![SharedMember {
                id: "id0".to_string(),
                member_class: "COM".to_string(),
                member_code: "5555".to_string(),
                name: "Roundtrip Member".to_string(),
                subsystems: vec![SharedSubsystem {
                    id: "id1".to_string(),
                    subsystem_code: "SUB1".to_string(),
                    subsystem_name: Some("Subsystem 1".to_string()),
                }],
            }],
            security_servers: vec![SharedSecurityServer {
                owner_id: "id0".to_string(),
                server_code: "SS5".to_string(),
                address: "https://ss5.example.com:8443".to_string(),
                auth_cert_hashes: vec!["hash123".to_string()],
                client_ids: vec!["id1".to_string()],
            }],
            global_groups: vec![],
            ocsp_freshness_seconds: Some(7200),
        };

        let xml = params.to_xml();
        let parsed = SharedParams::parse_xml(&xml).expect("Failed to parse generated xml");
        assert_eq!(parsed.instance_identifier, "DEV");
        assert_eq!(parsed.members.len(), 1);
        assert_eq!(parsed.members[0].member_code, "5555");
        assert_eq!(parsed.members[0].subsystems.len(), 1);
        assert_eq!(parsed.security_servers.len(), 1);
        assert_eq!(parsed.security_servers[0].server_code, "SS5");
    }
}
