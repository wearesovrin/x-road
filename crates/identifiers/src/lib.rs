pub mod central_service_id;
pub mod client_id;
pub mod group_id;
pub mod id_trait;
pub mod object_type;
pub mod server_id;
pub mod service_id;
pub mod validator;
pub mod xml;

pub use central_service_id::CentralServiceId;
pub use client_id::ClientId;
pub use group_id::{GlobalGroupId, LocalGroupId};
pub use id_trait::XRoadId;
pub use object_type::XRoadObjectType;
pub use server_id::SecurityServerId;
pub use service_id::ServiceId;
pub use validator::{MAX_IDENTIFIER_PART_LENGTH, StrictIdentifierValidator};
pub use xml::{
    XROAD_IDENTIFIERS_NS, XRoadClientIdentifierType, XRoadSecurityServerIdentifierType,
    XRoadServiceIdentifierType,
};

// Re-export common result and error
pub use common::{XRoadError, XRoadResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_identifier_validator() {
        // Valid strings
        assert!(StrictIdentifierValidator::is_valid("EE"));
        assert!(StrictIdentifierValidator::is_valid("COM"));
        assert!(StrictIdentifierValidator::is_valid("12345678"));
        assert!(StrictIdentifierValidator::is_valid("subsystem-1"));
        assert!(StrictIdentifierValidator::is_valid("getInvoices.v1"));
        assert!(StrictIdentifierValidator::is_valid("test'()+,.-=?"));

        // Invalid characters
        assert!(!StrictIdentifierValidator::is_valid(""));
        assert!(!StrictIdentifierValidator::is_valid("has space"));
        assert!(!StrictIdentifierValidator::is_valid("slash/part"));
        assert!(!StrictIdentifierValidator::is_valid("colon:part"));
        assert!(!StrictIdentifierValidator::is_valid("percent%"));
        assert!(!StrictIdentifierValidator::is_valid("ampersand&"));
        assert!(!StrictIdentifierValidator::is_valid("unicode_ö"));

        // Length limit (255 max)
        let exact_255 = "a".repeat(255);
        let overflow_256 = "a".repeat(256);
        assert!(StrictIdentifierValidator::is_valid(&exact_255));
        assert!(!StrictIdentifierValidator::is_valid(&overflow_256));

        assert!(StrictIdentifierValidator::validate_part("test", &exact_255).is_ok());
        assert!(StrictIdentifierValidator::validate_part("test", &overflow_256).is_err());
    }

    #[test]
    fn test_client_id_member_and_subsystem() {
        let member = ClientId::member("EE", "COM", "12345678").unwrap();
        assert_eq!(member.object_type(), XRoadObjectType::Member);
        assert!(member.is_member());
        assert!(!member.is_subsystem());
        assert_eq!(member.to_string(), "EE/COM/12345678");
        assert_eq!(member.to_encoded_id(true), "MEMBER:EE/COM/12345678");

        let subsystem = ClientId::subsystem("EE", "COM", "12345678", "accounting").unwrap();
        assert_eq!(subsystem.object_type(), XRoadObjectType::Subsystem);
        assert!(!subsystem.is_member());
        assert!(subsystem.is_subsystem());
        assert_eq!(subsystem.to_string(), "EE/COM/12345678/accounting");
        assert_eq!(
            subsystem.to_encoded_id(true),
            "SUBSYSTEM:EE/COM/12345678/accounting"
        );

        // member_id() of subsystem extracts parent member
        assert_eq!(subsystem.member_id(), member);
    }

    #[test]
    fn test_client_id_member_equals_and_subsystem_contains() {
        let member = ClientId::member("EE", "COM", "12345678").unwrap();
        let sub1 = ClientId::subsystem("EE", "COM", "12345678", "accounting").unwrap();
        let sub2 = ClientId::subsystem("EE", "COM", "12345678", "hr").unwrap();
        let other_member = ClientId::member("EE", "GOV", "12345678").unwrap();

        // Member equality across subsystems
        assert!(sub1.member_equals(&sub2));
        assert!(sub1.member_equals(&member));
        assert!(!sub1.member_equals(&other_member));

        // Subsystem contains member
        assert!(sub1.subsystem_contains_member(&member));
        assert!(!member.subsystem_contains_member(&sub1));
        assert!(!sub1.subsystem_contains_member(&other_member));
    }

    #[test]
    fn test_client_id_from_str() {
        let m: ClientId = "EE/COM/12345678".parse().unwrap();
        assert!(m.is_member());
        assert_eq!(m.member_code, "12345678");

        let m_prefixed: ClientId = "MEMBER:EE/COM/12345678".parse().unwrap();
        assert_eq!(m, m_prefixed);

        let sub: ClientId = "EE/COM/12345678/billing".parse().unwrap();
        assert!(sub.is_subsystem());
        assert_eq!(sub.subsystem_code.as_deref(), Some("billing"));

        let sub_prefixed: ClientId = "SUBSYSTEM:EE/COM/12345678/billing".parse().unwrap();
        assert_eq!(sub, sub_prefixed);

        // Errors on invalid prefixes or formats
        assert!("MEMBER:EE/COM/12345678/extra".parse::<ClientId>().is_err());
        assert!("SUBSYSTEM:EE/COM/12345678".parse::<ClientId>().is_err());
        assert!("EE/COM".parse::<ClientId>().is_err());
    }

    #[test]
    fn test_service_id_parsing_and_versions() {
        // Without subsystem: INSTANCE/CLASS/CODE/SERVICE
        let svc1: ServiceId = "EE/BUSINESS/JODA/getState".parse().unwrap();
        assert_eq!(svc1.xroad_instance(), "EE");
        assert_eq!(svc1.member_class(), "BUSINESS");
        assert_eq!(svc1.member_code(), "JODA");
        assert_eq!(svc1.subsystem_code(), None);
        assert_eq!(svc1.service_code(), "getState");
        assert_eq!(svc1.service_version(), None);
        assert_eq!(svc1.to_string(), "EE/BUSINESS/JODA/getState");

        // With subsystem: INSTANCE/CLASS/CODE/SUBSYSTEM/SERVICE
        let svc2: ServiceId = "EE/COM/12345678/billing/getInvoices".parse().unwrap();
        assert_eq!(svc2.subsystem_code(), Some("billing"));
        assert_eq!(svc2.service_code(), "getInvoices");
        assert_eq!(svc2.service_version(), None);

        // With subsystem and version: INSTANCE/CLASS/CODE/SUBSYSTEM/SERVICE/VERSION
        let svc3: ServiceId = "SERVICE:EE/COM/12345678/billing/getInvoices/v2"
            .parse()
            .unwrap();
        assert_eq!(svc3.subsystem_code(), Some("billing"));
        assert_eq!(svc3.service_code(), "getInvoices");
        assert_eq!(svc3.service_version(), Some("v2"));
        assert_eq!(svc3.to_string(), "EE/COM/12345678/billing/getInvoices/v2");
    }

    #[test]
    fn test_security_server_id() {
        let server: SecurityServerId = "SERVER:EE/COM/12345678/ss1".parse().unwrap();
        assert_eq!(server.xroad_instance, "EE");
        assert_eq!(server.member_class, "COM");
        assert_eq!(server.member_code, "12345678");
        assert_eq!(server.server_code, "ss1");
        assert_eq!(server.to_string(), "EE/COM/12345678/ss1");

        let owner = server.owner();
        assert!(owner.is_member());
        assert_eq!(owner.to_string(), "EE/COM/12345678");
    }

    #[test]
    fn test_groups_and_central_service() {
        let global_grp: GlobalGroupId = "GLOBALGROUP:EE/security-officers".parse().unwrap();
        assert_eq!(global_grp.xroad_instance, "EE");
        assert_eq!(global_grp.group_code, "security-officers");
        assert_eq!(global_grp.to_string(), "EE/security-officers");

        let local_grp: LocalGroupId = "LOCALGROUP:administrators".parse().unwrap();
        assert_eq!(local_grp.group_code, "administrators");
        assert_eq!(local_grp.to_string(), "administrators");

        let central_svc: CentralServiceId = "CENTRALSERVICE:EE/populationQuery".parse().unwrap();
        assert_eq!(central_svc.xroad_instance, "EE");
        assert_eq!(central_svc.service_code, "populationQuery");
        assert_eq!(central_svc.to_string(), "EE/populationQuery");
    }

    #[test]
    fn test_xml_client_identifier_roundtrip() {
        let member = ClientId::member("EE", "COMPANY", "FOOBAR").unwrap();
        let xml_repr = XRoadClientIdentifierType::from(&member);
        let xml_str = xml_repr.to_xml("client");

        assert!(xml_str.contains(r#"id:objectType="MEMBER""#));
        assert!(xml_str.contains("<id:xRoadInstance>EE</id:xRoadInstance>"));
        assert!(xml_str.contains("<id:memberClass>COMPANY</id:memberClass>"));
        assert!(xml_str.contains("<id:memberCode>FOOBAR</id:memberCode>"));
        assert!(!xml_str.contains("<id:subsystemCode>"));

        let subsystem = ClientId::subsystem("EE", "COMPANY", "FOOBAR", "SUBSYSTEM").unwrap();
        let sub_xml_repr = XRoadClientIdentifierType::from(&subsystem);
        let sub_xml_str = sub_xml_repr.to_xml("client");
        assert!(sub_xml_str.contains(r#"id:objectType="SUBSYSTEM""#));
        assert!(sub_xml_str.contains("<id:subsystemCode>SUBSYSTEM</id:subsystemCode>"));
    }

    #[test]
    fn test_xml_service_identifier_roundtrip() {
        let svc = ServiceId::from_parts(
            "EE",
            "BUSINESS",
            "JODA",
            None::<&str>,
            "getState",
            Some("v1"),
        )
        .unwrap();
        let xml_repr = XRoadServiceIdentifierType::from(&svc);
        let xml_str = xml_repr.to_xml("service");

        assert!(xml_str.contains(r#"id:objectType="SERVICE""#));
        assert!(xml_str.contains("<id:serviceCode>getState</id:serviceCode>"));
        assert!(xml_str.contains("<id:serviceVersion>v1</id:serviceVersion>"));
    }

    #[test]
    fn test_json_serialization() {
        let client = ClientId::subsystem("EE", "COM", "12345", "sub1").unwrap();
        let json = serde_json::to_string(&client).unwrap();
        assert!(json.contains("xRoadInstance"));
        assert!(json.contains("subsystemCode"));

        let deserialized: ClientId = serde_json::from_str(&json).unwrap();
        assert_eq!(client, deserialized);
    }
}
