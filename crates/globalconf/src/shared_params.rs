use base64::prelude::*;
use common::{XRoadError, XRoadResult};
use identifiers::ClientId;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

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

/// Complete in-memory representation of `shared-params.xml`.
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
    /// Parses a `shared-params.xml` string into a `SharedParams` struct.
    pub fn parse_xml(xml: &str) -> XRoadResult<Self> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut params = SharedParams::default();
        let mut buf = Vec::new();

        // Temporary tracking states
        let mut tag_stack: Vec<String> = Vec::new();
        let mut current_member: Option<SharedMember> = None;
        let mut current_subsystem: Option<SharedSubsystem> = None;
        let mut current_sec_server: Option<SharedSecurityServer> = None;
        let mut current_ca: Option<SharedApprovedCa> = None;
        let mut current_tsa: Option<SharedApprovedTsa> = None;
        let mut current_group: Option<SharedGlobalGroup> = None;
        let mut current_group_member_instance: Option<String> = None;
        let mut current_group_member_class: Option<String> = None;
        let mut current_group_member_code: Option<String> = None;
        let mut current_group_member_subsystem: Option<String> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let local_tag = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                    match local_tag.as_str() {
                        "member" => {
                            let mut id = String::new();
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"id" {
                                    id = String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                            current_member = Some(SharedMember {
                                id,
                                member_class: String::new(),
                                member_code: String::new(),
                                name: String::new(),
                                subsystems: Vec::new(),
                            });
                        }
                        "subsystem" => {
                            let mut id = String::new();
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"id" {
                                    id = String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                            current_subsystem = Some(SharedSubsystem {
                                id,
                                subsystem_code: String::new(),
                                subsystem_name: None,
                            });
                        }
                        "securityServer" => {
                            current_sec_server = Some(SharedSecurityServer {
                                owner_id: String::new(),
                                server_code: String::new(),
                                address: String::new(),
                                auth_cert_hashes: Vec::new(),
                                client_ids: Vec::new(),
                            });
                        }
                        "approvedCA" => {
                            current_ca = Some(SharedApprovedCa {
                                name: String::new(),
                                authentication_only: false,
                                top_ca_cert: Vec::new(),
                                ocsp_url: None,
                                ocsp_cert: None,
                                certificate_profile_info: None,
                                acme_server_url: None,
                            });
                        }
                        "approvedTSA" => {
                            current_tsa = Some(SharedApprovedTsa {
                                name: String::new(),
                                url: String::new(),
                                cert: Vec::new(),
                            });
                        }
                        "globalGroup" => {
                            current_group = Some(SharedGlobalGroup {
                                group_code: String::new(),
                                description: String::new(),
                                members: Vec::new(),
                            });
                        }
                        "groupMember" => {
                            current_group_member_instance = None;
                            current_group_member_class = None;
                            current_group_member_code = None;
                            current_group_member_subsystem = None;
                        }
                        _ => {}
                    }

                    tag_stack.push(local_tag);
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if text.is_empty() {
                        continue;
                    }

                    let parent_tag = tag_stack.last().map(|s| s.as_str()).unwrap_or("");
                    let grand_parent_tag = if tag_stack.len() >= 2 {
                        tag_stack.get(tag_stack.len() - 2).map(|s| s.as_str()).unwrap_or("")
                    } else {
                        ""
                    };

                    match parent_tag {
                        "instanceIdentifier" => {
                            params.instance_identifier = text;
                        }
                        "ocspFreshnessSeconds" => {
                            params.ocsp_freshness_seconds = text.parse().ok();
                        }
                        // Member & Subsystem
                        "code" if grand_parent_tag == "memberClass" => {
                            if let Some(m) = &mut current_member {
                                m.member_class = text;
                            }
                        }
                        "memberCode" => {
                            if let Some(m) = &mut current_member {
                                m.member_code = text;
                            } else if tag_stack.iter().any(|t| t == "groupMember") {
                                current_group_member_code = Some(text);
                            }
                        }
                        "name" => {
                            if grand_parent_tag == "member" {
                                if let Some(m) = &mut current_member {
                                    m.name = text;
                                }
                            } else if grand_parent_tag == "approvedCA" {
                                if let Some(ca) = &mut current_ca {
                                    ca.name = text;
                                }
                            } else if grand_parent_tag == "approvedTSA" {
                                if let Some(tsa) = &mut current_tsa {
                                    tsa.name = text;
                                }
                            }
                        }
                        "subsystemCode" => {
                            if let Some(s) = &mut current_subsystem {
                                s.subsystem_code = text;
                            } else if tag_stack.iter().any(|t| t == "groupMember") {
                                current_group_member_subsystem = Some(text);
                            }
                        }
                        "subsystemName" => {
                            if let Some(s) = &mut current_subsystem {
                                s.subsystem_name = Some(text);
                            }
                        }
                        // Security Server
                        "owner" if grand_parent_tag == "securityServer" => {
                            if let Some(ss) = &mut current_sec_server {
                                ss.owner_id = text;
                            }
                        }
                        "serverCode" if grand_parent_tag == "securityServer" => {
                            if let Some(ss) = &mut current_sec_server {
                                ss.server_code = text;
                            }
                        }
                        "address" if grand_parent_tag == "securityServer" => {
                            if let Some(ss) = &mut current_sec_server {
                                ss.address = text;
                            }
                        }
                        "authCertHash" => {
                            if let Some(ss) = &mut current_sec_server {
                                ss.auth_cert_hashes.push(text);
                            }
                        }
                        "client" if grand_parent_tag == "securityServer" => {
                            if let Some(ss) = &mut current_sec_server {
                                ss.client_ids.push(text);
                            }
                        }
                        // CA & TSA
                        "authenticationOnly" => {
                            if let Some(ca) = &mut current_ca {
                                ca.authentication_only = text.eq_ignore_ascii_case("true");
                            }
                        }
                        "cert" => {
                            let clean_text = text.replace(['\r', '\n', ' '], "");
                            let decoded = BASE64_STANDARD.decode(&clean_text).unwrap_or_else(|_| text.as_bytes().to_vec());
                            if grand_parent_tag == "topCA" {
                                if let Some(ca) = &mut current_ca {
                                    ca.top_ca_cert = decoded;
                                }
                            } else if grand_parent_tag == "ocsp" {
                                if let Some(ca) = &mut current_ca {
                                    ca.ocsp_cert = Some(decoded);
                                }
                            } else if grand_parent_tag == "approvedTSA" {
                                if let Some(tsa) = &mut current_tsa {
                                    tsa.cert = decoded;
                                }
                            }
                        }
                        "url" => {
                            if grand_parent_tag == "ocsp" {
                                if let Some(ca) = &mut current_ca {
                                    ca.ocsp_url = Some(text);
                                }
                            } else if grand_parent_tag == "approvedTSA" {
                                if let Some(tsa) = &mut current_tsa {
                                    tsa.url = text;
                                }
                            }
                        }
                        "certificateProfileInfo" => {
                            if let Some(ca) = &mut current_ca {
                                ca.certificate_profile_info = Some(text);
                            }
                        }
                        "directoryURL" => {
                            if let Some(ca) = &mut current_ca {
                                ca.acme_server_url = Some(text);
                            }
                        }
                        // Global Group
                        "groupCode" => {
                            if let Some(g) = &mut current_group {
                                g.group_code = text;
                            }
                        }
                        "description" if grand_parent_tag == "globalGroup" => {
                            if let Some(g) = &mut current_group {
                                g.description = text;
                            }
                        }
                        "xRoadInstance" => {
                            if tag_stack.iter().any(|t| t == "groupMember") {
                                current_group_member_instance = Some(text);
                            }
                        }
                        "memberClass" if tag_stack.iter().any(|t| t == "groupMember") => {
                            current_group_member_class = Some(text);
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let local_tag = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                    match local_tag.as_str() {
                        "subsystem" => {
                            if let (Some(sub), Some(m)) = (current_subsystem.take(), &mut current_member) {
                                m.subsystems.push(sub);
                            }
                        }
                        "member" => {
                            if let Some(m) = current_member.take() {
                                params.members.push(m);
                            }
                        }
                        "securityServer" => {
                            if let Some(ss) = current_sec_server.take() {
                                params.security_servers.push(ss);
                            }
                        }
                        "approvedCA" => {
                            if let Some(ca) = current_ca.take() {
                                params.approved_cas.push(ca);
                            }
                        }
                        "approvedTSA" => {
                            if let Some(tsa) = current_tsa.take() {
                                params.approved_tsas.push(tsa);
                            }
                        }
                        "groupMember" => {
                            if let (Some(inst), Some(cls), Some(code)) = (
                                current_group_member_instance.take(),
                                current_group_member_class.take(),
                                current_group_member_code.take(),
                            ) {
                                let client_id = match current_group_member_subsystem.take() {
                                    Some(sub) => ClientId::subsystem_unchecked(inst, cls, code, sub),
                                    None => ClientId::member_unchecked(inst, cls, code),
                                };
                                if let Some(g) = &mut current_group {
                                    g.members.push(client_id);
                                }
                            }
                        }
                        "globalGroup" => {
                            if let Some(g) = current_group.take() {
                                params.global_groups.push(g);
                            }
                        }
                        _ => {}
                    }

                    tag_stack.pop();
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(XRoadError::Configuration(format!("XML parse error: {e}"))),
                _ => {}
            }
            buf.clear();
        }

        Ok(params)
    }

    /// Serializes the `SharedParams` into a standard X-Road `shared-params.xml` string.
    pub fn to_xml(&self) -> String {
        let mut out = String::new();
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        out.push_str("<ns3:conf xmlns:ns2=\"http://x-road.eu/xsd/identifiers\" xmlns:ns3=\"http://x-road.eu/xsd/xroad.xsd\">\n");
        out.push_str(&format!("    <instanceIdentifier>{}</instanceIdentifier>\n", self.instance_identifier));

        // Approved CAs
        for ca in &self.approved_cas {
            out.push_str("    <approvedCA>\n");
            out.push_str(&format!("        <name>{}</name>\n", escape_xml(&ca.name)));
            out.push_str(&format!("        <authenticationOnly>{}</authenticationOnly>\n", ca.authentication_only));
            out.push_str("        <topCA>\n");
            out.push_str(&format!("            <cert>{}</cert>\n", BASE64_STANDARD.encode(&ca.top_ca_cert)));
            if let Some(ocsp_url) = &ca.ocsp_url {
                out.push_str("            <ocsp>\n");
                out.push_str(&format!("                <url>{}</url>\n", escape_xml(ocsp_url)));
                if let Some(cert) = &ca.ocsp_cert {
                    out.push_str(&format!("                <cert>{}</cert>\n", BASE64_STANDARD.encode(cert)));
                }
                out.push_str("                <costType>FREE</costType>\n");
                out.push_str("            </ocsp>\n");
            }
            out.push_str("        </topCA>\n");
            if let Some(profile) = &ca.certificate_profile_info {
                out.push_str(&format!("        <certificateProfileInfo>{}</certificateProfileInfo>\n", escape_xml(profile)));
            }
            if let Some(acme) = &ca.acme_server_url {
                out.push_str("        <acmeServer>\n");
                out.push_str(&format!("            <directoryURL>{}</directoryURL>\n", escape_xml(acme)));
                out.push_str("        </acmeServer>\n");
            }
            out.push_str("    </approvedCA>\n");
        }

        // Approved TSAs
        for tsa in &self.approved_tsas {
            out.push_str("    <approvedTSA>\n");
            out.push_str(&format!("        <name>{}</name>\n", escape_xml(&tsa.name)));
            out.push_str(&format!("        <url>{}</url>\n", escape_xml(&tsa.url)));
            out.push_str(&format!("        <cert>{}</cert>\n", BASE64_STANDARD.encode(&tsa.cert)));
            out.push_str("        <costType>FREE</costType>\n");
            out.push_str("    </approvedTSA>\n");
        }

        // Members
        for m in &self.members {
            out.push_str(&format!("    <member id=\"{}\">\n", escape_xml(&m.id)));
            out.push_str("        <memberClass>\n");
            out.push_str(&format!("            <code>{}</code>\n", escape_xml(&m.member_class)));
            out.push_str("        </memberClass>\n");
            out.push_str(&format!("        <memberCode>{}</memberCode>\n", escape_xml(&m.member_code)));
            out.push_str(&format!("        <name>{}</name>\n", escape_xml(&m.name)));
            for sub in &m.subsystems {
                out.push_str(&format!("        <subsystem id=\"{}\">\n", escape_xml(&sub.id)));
                out.push_str(&format!("            <subsystemCode>{}</subsystemCode>\n", escape_xml(&sub.subsystem_code)));
                if let Some(sub_name) = &sub.subsystem_name {
                    out.push_str(&format!("            <subsystemName>{}</subsystemName>\n", escape_xml(sub_name)));
                }
                out.push_str("        </subsystem>\n");
            }
            out.push_str("    </member>\n");
        }

        // Security Servers
        for ss in &self.security_servers {
            out.push_str("    <securityServer>\n");
            out.push_str(&format!("        <owner>{}</owner>\n", escape_xml(&ss.owner_id)));
            out.push_str(&format!("        <serverCode>{}</serverCode>\n", escape_xml(&ss.server_code)));
            out.push_str(&format!("        <address>{}</address>\n", escape_xml(&ss.address)));
            for hash in &ss.auth_cert_hashes {
                out.push_str(&format!("        <authCertHash>{}</authCertHash>\n", escape_xml(hash)));
            }
            for client_ref in &ss.client_ids {
                out.push_str(&format!("        <client>{}</client>\n", escape_xml(client_ref)));
            }
            out.push_str("    </securityServer>\n");
        }

        // Global Groups
        for g in &self.global_groups {
            out.push_str("    <globalGroup>\n");
            out.push_str(&format!("        <groupCode>{}</groupCode>\n", escape_xml(&g.group_code)));
            out.push_str(&format!("        <description>{}</description>\n", escape_xml(&g.description)));
            for member in &g.members {
                let obj_type = if member.subsystem_code.is_some() { "SUBSYSTEM" } else { "MEMBER" };
                out.push_str(&format!("        <groupMember ns2:objectType=\"{}\">\n", obj_type));
                out.push_str(&format!("            <ns2:xRoadInstance>{}</ns2:xRoadInstance>\n", escape_xml(&member.xroad_instance)));
                out.push_str(&format!("            <ns2:memberClass>{}</ns2:memberClass>\n", escape_xml(&member.member_class)));
                out.push_str(&format!("            <ns2:memberCode>{}</ns2:memberCode>\n", escape_xml(&member.member_code)));
                if let Some(sub) = &member.subsystem_code {
                    out.push_str(&format!("            <ns2:subsystemCode>{}</ns2:subsystemCode>\n", escape_xml(sub)));
                }
                out.push_str("        </groupMember>\n");
            }
            out.push_str("    </globalGroup>\n");
        }

        // Global Settings
        out.push_str("    <globalSettings>\n");
        if let Some(sec) = self.ocsp_freshness_seconds {
            out.push_str(&format!("        <ocspFreshnessSeconds>{}</ocspFreshnessSeconds>\n", sec));
        }
        out.push_str("    </globalSettings>\n");

        out.push_str("</ns3:conf>\n");
        out
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
